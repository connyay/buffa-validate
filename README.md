# buffa-validate

Rust implementation of [protovalidate](https://buf.build/docs/protovalidate/) for [buffa](https://github.com/anthropics/buffa)-generated types. Reads `buf.validate.*` proto annotations and generates compile-time validation code, with optional [connect-rust](https://github.com/anthropics/connect-rust) integration.

## Crates

| Crate                    | Purpose                                                                    |
| ------------------------ | -------------------------------------------------------------------------- |
| `buffa-validate`         | Runtime library: `Validate` trait, `Violations` type, ConnectRPC helpers   |
| `buffa-validate-codegen` | Code generation library (used by build crate)                              |
| `buffa-validate-build`   | `build.rs` post-pass that adds validation to existing buffa/connectrpc output |

## Quick start

### 1. Add dependencies

```toml
# Cargo.toml
[dependencies]
buffa = "0.6"
buffa-validate = { version = "0.1", features = ["connectrpc"] }
connectrpc = "0.6"

[build-dependencies]
connectrpc-build = "0.6"
buffa-validate-build = "0.1"
```

### 2. Write your build.rs

`buffa-validate-build` runs as a post-pass after your normal buffa or connect-rust build. It generates `impl Validate` companions and patches the existing module tree to include them.

```rust
fn main() {
    // Bundled buf/validate/validate.proto — pass to any build crate's includes
    let validate_include = buffa_validate_build::include_dir();

    // Step 1: normal connect-rust build (or buffa-build)
    connectrpc_build::Config::new()
        .files(&["proto/service.proto"])
        .includes(&["proto/", validate_include.to_str().unwrap()])
        .include_file("_include.rs")
        .compile()
        .unwrap();

    // Step 2: add validation impls
    buffa_validate_build::Config::new()
        .files(&["proto/service.proto"])
        .includes(&["proto/"])
        .compile()
        .unwrap();
}
```

### 3. Annotate your protos

```protobuf
syntax = "proto3";
import "buf/validate/validate.proto";

message CreateUserRequest {
  string name = 1 [(buf.validate.field).string.min_len = 1];
  string email = 2 [(buf.validate.field).string.email = true];
  uint32 age = 3 [(buf.validate.field).uint32 = { gte: 0, lte: 150 }];
}
```

### 4. Validate

```rust
use buffa_validate::Validate;

let req = CreateUserRequest {
    name: "".into(),
    email: "not-an-email".into(),
    age: 200,
    ..Default::default()
};

let err = req.validate().unwrap_err();
for v in &err.violations {
    println!("{}: {} [{}]", v.field_path, v.message, v.rule);
}
// name: value length must be at least 1 characters [string.min_len]
// email: value must be a valid email address [string.email]
// age: value must be less than or equal to 150 [uint32.lte]
```

## Connect-rust integration

Enable the `connectrpc` feature on `buffa-validate` to get the `ValidateExt` trait.

`.validated()` is an extension method on any type implementing `Validate`. Returns `Result<&Self, ConnectError>` with structured error details:

```rust
use buffa_validate::ValidateExt;

async fn create_user(
    &self,
    ctx: connectrpc::RequestContext,
    req: OwnedCreateUserRequestView,
) -> connectrpc::ServiceResult<impl Encodable<CreateUserResponse> + Send> {
    req.validated()?; // returns ConnectError::InvalidArgument on failure
    // ...
}
```

Auto-deref means this works directly on `OwnedView<FooView<'static>>` (connect-rust's request type) without any conversion.

## Supported constraints

### Scalar types

- **String**: `const`, `len`, `min_len`, `max_len`, `len_bytes`, `min_bytes`, `max_bytes`, `pattern`, `prefix`, `suffix`, `contains`, `not_contains`, `in`, `not_in`, well-known formats (email, hostname, ip, ipv4, ipv6, uri, uuid, address, host_and_port, ip_with_prefixlen, ip_prefix, etc.)
- **Numeric** (int32/int64/uint32/uint64/sint32/sint64/fixed32/fixed64/sfixed32/sfixed64/float/double): `const`, `lt`, `lte`, `gt`, `gte`, `in`, `not_in`, `finite`
- **Bool**: `const`
- **Bytes**: `const`, `len`, `min_len`, `max_len`, `prefix`, `suffix`, `contains`, `in`, `not_in`
- **Enum**: `const`, `defined_only`, `in`, `not_in`

### Composite types

- **Repeated**: `min_items`, `max_items`, `unique`, `items` (nested field rules)
- **Map**: `min_pairs`, `max_pairs`, `keys` (nested rules), `values` (nested rules)
- **Message**: recursive validation of nested messages with field path propagation

### Well-known types

- **Duration**: `const`, `lt`, `lte`, `gt`, `gte`, `in`, `not_in`
- **Timestamp**: `const`, `lt`, `lte`, `gt`, `gte`, `lt_now`, `gt_now`, `within`
- **Any**: `in`, `not_in` (type URL matching)

### Other

- **Oneof**: `required`
- **CEL expressions**: field-level and message-level custom validation via the `cel` rule
- **Ignore**: `IGNORE_ALWAYS`, `IGNORE_IF_ZERO_VALUE`
- **Required**: works on optional fields and message fields

## View type support

Validation is generated for both owned types and buffa view types. `FooView<'a>` validates identically to `Foo`, and `OwnedView<FooView<'static>>` (connect-rust's request type) can be validated in place without converting to an owned message.

## Architecture

`buffa-validate-build` is a post-processing step that composes with your existing build:

1. Your upstream build (`connectrpc-build` or `buffa-build`) generates message types, service traits, and module stitchers as usual
2. `buffa-validate-build` acquires the same descriptors via protoc/buf
3. `buffa-validate-codegen` reads `buf.validate.*` extensions from field/message/oneof options and generates `impl Validate` blocks in companion `.__validate.rs` files
4. The companion files are written to `$OUT_DIR` and the existing stitcher files are patched to include them
