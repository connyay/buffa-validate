use std::fmt;

#[derive(Debug, Clone)]
pub struct Violations {
    pub violations: Vec<Violation>,
}

impl Violations {
    pub fn is_empty(&self) -> bool {
        self.violations.is_empty()
    }

    pub fn len(&self) -> usize {
        self.violations.len()
    }
}

impl fmt::Display for Violations {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, v) in self.violations.iter().enumerate() {
            if i > 0 {
                write!(f, "; ")?;
            }
            write!(f, "{v}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Violations {}

#[derive(Debug, Clone)]
pub struct Violation {
    pub field_path: String,
    pub rule: String,
    pub constraint_id: String,
    pub message: String,
}

impl Violation {
    pub fn new(
        field_path: impl Into<String>,
        rule: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        let rule = rule.into();
        Self {
            field_path: field_path.into(),
            constraint_id: rule.clone(),
            rule,
            message: message.into(),
        }
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} [{}]", self.field_path, self.message, self.rule)
    }
}
