//! Explicit synthetic legacy configuration for prototype unit tests only.
//! No fixture can populate the production organization registry.

use std::ops::{Deref, DerefMut};
use std::path::Path;

use repo_model::{Model, ModelError, OwnerInputs};

/// Prototype test context; its owner record is fictional and grants no authority.
pub struct SyntheticModel {
    model: Model,
    /// Synthetic organization input used to exercise configuration validators.
    pub owner_inputs: OwnerInputs,
}

impl SyntheticModel {
    /// Load real platform configuration and an explicitly synthetic test record.
    pub fn load(directory: &Path) -> Result<Self, ModelError> {
        let path = repo_model::repo_root().join("tests/fixtures/synthetic-owner-inputs.toml");
        let source = std::fs::read_to_string(&path).map_err(|error| ModelError::Io(path, error))?;
        let mut model = Model::load(directory)?;
        for boundary in &mut model.implementation_closure.accepted_boundaries {
            boundary.status = "accepted".to_string();
        }
        Ok(Self {
            model,
            owner_inputs: OwnerInputs::parse_toml(&source)?,
        })
    }

    /// Check prototype configuration; this is not product or authority acceptance.
    pub fn check(&self) -> Result<(), ModelError> {
        self.model.check()?;
        self.model
            .check_organization_configuration(&self.owner_inputs)
    }
}

impl Deref for SyntheticModel {
    type Target = Model;

    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

impl DerefMut for SyntheticModel {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.model
    }
}
