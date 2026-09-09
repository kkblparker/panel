#![allow(clippy::default_constructed_unit_structs)]
#![allow(unused_imports)]

use shared::extensions::{ConstructedExtension, distr::MetadataToml};
use std::sync::Arc;

pub fn list() -> Vec<ConstructedExtension> {
    vec![
        ConstructedExtension {
            metadata_toml: MetadataToml {
                package_name: "dev.kkblparker.pfsensesync".to_string(),
                name: "pfSense Firewall Sync".to_string(),
                panel_version: semver::VersionReq::parse(">=1.1.0").unwrap(),
                license_text: None,
            },
            package_name: "dev.kkblparker.pfsensesync",
            description: "Automatically syncs Calagopus server allocations to pfSense NAT port-forward rules via the pfSense REST API.",
            authors: &["kkblparker"],
            version: semver::Version::parse("1.3.0").unwrap(),
            extension: Arc::new(dev_kkblparker_pfsensesync::ExtensionStruct::default()),
        },
        ConstructedExtension {
            metadata_toml: MetadataToml {
                package_name: "dev.kkblparker.reforgerworkshop".to_string(),
                name: "Arma Reforger Workshop".to_string(),
                panel_version: semver::VersionReq::parse(">=1.1.0").unwrap(),
                license_text: None,
            },
            package_name: "dev.kkblparker.reforgerworkshop",
            description: "Adds an in-panel Workshop browser for Arma Reforger servers, backed by the reforgermods.net API.",
            authors: &["kkblparker"],
            version: semver::Version::parse("1.0.0").unwrap(),
            extension: Arc::new(dev_kkblparker_reforgerworkshop::ExtensionStruct::default()),
        },
    ]
}
