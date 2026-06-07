//! HNF v0.1 save/load for FreeCAD mechanical domain.

use hnf_core::{
    parse_mechanical, serialize_mechanical, validate, HnfDocument, HnfManifest, HnfMutation,
    HnfObject, HNF_VERSION_V0_1, MECHANICAL_DOMAIN, MECHANICAL_VERSION,
};
use hnf_core::{
    MechanicalConstraint, MechanicalDomain, MechanicalGeometryBlob, MechanicalProperties,
    MechanicalSolid,
};
use hnf_core::domain::HNF_TYPE_OBJECT;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FreecadHnfError {
    #[error("manifest validation failed: {0}")]
    Manifest(String),
    #[error("domain parse failed: {0}")]
    DomainParse(String),
    #[error("missing mechanical object in HNF document")]
    MissingMechanical,
    #[error("serialization failed: {0}")]
    Serialization(String),
    #[error("deserialization failed: {0}")]
    Deserialization(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MechanicalSnapshot {
    #[serde(default)]
    pub solids: Vec<MechanicalSolidSnapshot>,
    #[serde(default)]
    pub constraints: Vec<MechanicalConstraintSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MechanicalSolidSnapshot {
    pub solid_id: String,
    pub name: String,
    #[serde(default)]
    pub material: String,
    #[serde(default)]
    pub volume_mm3: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_content_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MechanicalConstraintSnapshot {
    pub constraint_id: String,
    pub from_solid_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_solid_id: Option<String>,
    pub constraint_type: String,
}

pub fn mechanical_domain_from_snapshot(
    snapshot: &MechanicalSnapshot,
    object_id: &str,
) -> Result<MechanicalDomain, FreecadHnfError> {
    if snapshot.solids.is_empty() {
        return Err(FreecadHnfError::DomainParse(
            "mechanical snapshot requires at least one solid".into(),
        ));
    }

    let solids: Vec<MechanicalSolid> = snapshot
        .solids
        .iter()
        .map(|s| {
            let geometry_blobs = s
                .step_content_hash
                .as_ref()
                .filter(|h| h.len() == 64)
                .map(|h| {
                    vec![MechanicalGeometryBlob {
                        format: "step".into(),
                        content_hash: h.clone(),
                    }]
                })
                .unwrap_or_default();
            MechanicalSolid {
                id: s.solid_id.clone(),
                name: s.name.clone(),
                material: if s.material.is_empty() {
                    None
                } else {
                    Some(s.material.clone())
                },
                volume_mm3: Some(s.volume_mm3),
                geometry_blobs,
            }
        })
        .collect();

    let constraints: Vec<MechanicalConstraint> = snapshot
        .constraints
        .iter()
        .map(|c| MechanicalConstraint {
            id: c.constraint_id.clone(),
            constraint_type: c.constraint_type.clone(),
            solid_a: c.from_solid_id.clone(),
            solid_b: c.to_solid_id.clone(),
        })
        .collect();

    let domain = MechanicalDomain {
        domain: MECHANICAL_DOMAIN.into(),
        version: MECHANICAL_VERSION.into(),
        hnf_type: HNF_TYPE_OBJECT.into(),
        object_id: object_id.into(),
        content_hash: None,
        refs: vec![],
        properties: MechanicalProperties {
            solids,
            constraints,
            ..Default::default()
        },
    };

    parse_mechanical(&serialize_mechanical(&domain))
        .map_err(|e| FreecadHnfError::DomainParse(e.to_string()))
}

pub fn mechanical_snapshot_from_domain(domain: &MechanicalDomain) -> MechanicalSnapshot {
    MechanicalSnapshot {
        solids: domain
            .properties
            .solids
            .iter()
            .map(|s| MechanicalSolidSnapshot {
                solid_id: s.id.clone(),
                name: s.name.clone(),
                material: s.material.clone().unwrap_or_default(),
                volume_mm3: s.volume_mm3.unwrap_or(0.0),
                step_content_hash: s
                    .geometry_blobs
                    .iter()
                    .find(|b| b.format == "step")
                    .map(|b| b.content_hash.clone()),
            })
            .collect(),
        constraints: domain
            .properties
            .constraints
            .iter()
            .map(|c| MechanicalConstraintSnapshot {
                constraint_id: c.id.clone(),
                from_solid_id: c.solid_a.clone(),
                to_solid_id: c.solid_b.clone(),
                constraint_type: c.constraint_type.clone(),
            })
            .collect(),
    }
}

pub fn build_mechanical_hnf_document(
    snapshot: &MechanicalSnapshot,
    doc_id: &str,
    document_uri: &str,
) -> Result<HnfDocument, FreecadHnfError> {
    let object_id = format!("{doc_id}-mechanical");
    let domain = mechanical_domain_from_snapshot(snapshot, &object_id)?;
    Ok(HnfDocument {
        manifest: HnfManifest {
            hnf_version: HNF_VERSION_V0_1.to_string(),
            doc_id: doc_id.to_string(),
            disciplines: vec![MECHANICAL_DOMAIN.to_string()],
            created_at: None,
            schema_revision: None,
        },
        document_uri: document_uri.to_string(),
        metadata: json!({ "adapter": "hnf-freecad", "domain": MECHANICAL_DOMAIN }),
        objects: vec![HnfObject {
            id: object_id,
            kind: MECHANICAL_DOMAIN.to_string(),
            properties: serialize_mechanical(&domain),
        }],
    })
}

pub fn export_hnf_json(doc: &HnfDocument) -> Result<String, FreecadHnfError> {
    validate(&doc.manifest).map_err(|errs| {
        FreecadHnfError::Manifest(
            errs.iter()
                .map(|e| format!("{}: {}", e.field, e.message))
                .collect::<Vec<_>>()
                .join("; "),
        )
    })?;
    serde_json::to_string_pretty(doc)
        .map_err(|e| FreecadHnfError::Serialization(e.to_string()))
}

pub fn import_hnf_json(raw: &str) -> Result<HnfDocument, FreecadHnfError> {
    let doc: HnfDocument =
        serde_json::from_str(raw).map_err(|e| FreecadHnfError::Deserialization(e.to_string()))?;
    validate(&doc.manifest).map_err(|errs| {
        FreecadHnfError::Manifest(
            errs.iter()
                .map(|e| format!("{}: {}", e.field, e.message))
                .collect::<Vec<_>>()
                .join("; "),
        )
    })?;
    Ok(doc)
}

pub fn extract_mechanical_domain(doc: &HnfDocument) -> Result<MechanicalDomain, FreecadHnfError> {
    let obj = doc
        .objects
        .iter()
        .find(|o| o.kind == MECHANICAL_DOMAIN)
        .ok_or(FreecadHnfError::MissingMechanical)?;
    parse_mechanical(&obj.properties).map_err(|e| FreecadHnfError::DomainParse(e.to_string()))
}

pub fn mechanical_roundtrip_fingerprint(snapshot: &MechanicalSnapshot) -> Result<String, FreecadHnfError> {
    let doc = build_mechanical_hnf_document(snapshot, "roundtrip-doc", "hbp://roundtrip/mechanical")?;
    let json = export_hnf_json(&doc)?;
    let imported = import_hnf_json(&json)?;
    let domain = extract_mechanical_domain(&imported)?;
    let replay = mechanical_snapshot_from_domain(&domain);
    Ok(format!(
        "{}|{}|{}",
        replay.solids.len(),
        replay.constraints.len(),
        replay
            .solids
            .first()
            .map(|s| s.solid_id.as_str())
            .unwrap_or("")
    ))
}

fn doc_id_from_project(project_path: &str) -> String {
    project_path
        .rsplit('/')
        .next()
        .unwrap_or(project_path)
        .replace('.', "-")
}

pub fn mutations_to_mechanical_snapshot(
    mutations: &[HnfMutation],
) -> Result<MechanicalSnapshot, FreecadHnfError> {
    let mut solids = Vec::new();
    let mut constraints = Vec::new();
    for m in mutations {
        match m.kind.as_str() {
            "mechanical/solid/upsert" => {
                solids.push(MechanicalSolidSnapshot {
                    solid_id: m
                        .payload
                        .get("solid_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("solid-unknown")
                        .to_string(),
                    name: m
                        .payload
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unnamed")
                        .to_string(),
                    material: m
                        .payload
                        .get("material")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    volume_mm3: m
                        .payload
                        .get("volume_mm3")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0),
                    step_content_hash: m
                        .payload
                        .get("step_content_hash")
                        .and_then(|v| v.as_str())
                        .map(str::to_string),
                });
            }
            "mechanical/constraint/upsert" => {
                constraints.push(MechanicalConstraintSnapshot {
                    constraint_id: m
                        .payload
                        .get("constraint_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("constraint-unknown")
                        .to_string(),
                    from_solid_id: m
                        .payload
                        .get("from_solid_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    to_solid_id: m
                        .payload
                        .get("to_solid_id")
                        .and_then(|v| v.as_str())
                        .map(str::to_string),
                    constraint_type: m
                        .payload
                        .get("constraint_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("mate")
                        .to_string(),
                });
            }
            other => {
                return Err(FreecadHnfError::DomainParse(format!(
                    "unsupported mechanical mutation kind: {other}"
                )));
            }
        }
    }
    Ok(MechanicalSnapshot { solids, constraints })
}

pub fn mechanical_snapshot_to_mutations(snapshot: &MechanicalSnapshot) -> Vec<HnfMutation> {
    let mut out = Vec::new();
    for solid in &snapshot.solids {
        out.push(HnfMutation {
            kind: "mechanical/solid/upsert".into(),
            payload: json!({
                "commitId": "freecad-import",
                "solid_id": solid.solid_id,
                "name": solid.name,
                "material": solid.material,
                "volume_mm3": solid.volume_mm3,
                "step_content_hash": solid.step_content_hash,
            }),
        });
    }
    for c in &snapshot.constraints {
        out.push(HnfMutation {
            kind: "mechanical/constraint/upsert".into(),
            payload: json!({
                "commitId": "freecad-import",
                "constraint_id": c.constraint_id,
                "from_solid_id": c.from_solid_id,
                "to_solid_id": c.to_solid_id,
                "constraint_type": c.constraint_type,
            }),
        });
    }
    out
}

pub fn mutations_to_mechanical_domain(
    project_path: &str,
    mutations: &[HnfMutation],
) -> Result<serde_json::Value, FreecadHnfError> {
    let snap = mutations_to_mechanical_snapshot(mutations)?;
    let doc_id = doc_id_from_project(project_path);
    let doc = build_mechanical_hnf_document(&snap, &doc_id, project_path)?;
    serde_json::to_value(doc).map_err(|e| FreecadHnfError::Serialization(e.to_string()))
}

pub fn mechanical_domain_to_mutations(
    hnf_document: &serde_json::Value,
) -> Result<Vec<HnfMutation>, FreecadHnfError> {
    let doc: HnfDocument = serde_json::from_value(hnf_document.clone())
        .map_err(|e| FreecadHnfError::Deserialization(e.to_string()))?;
    let domain = extract_mechanical_domain(&doc)?;
    Ok(mechanical_snapshot_to_mutations(&mechanical_snapshot_from_domain(
        &domain,
    )))
}

pub fn roundtrip_mechanical(
    project_path: &str,
    mutations: &[HnfMutation],
) -> Result<(), FreecadHnfError> {
    let exported = mutations_to_mechanical_domain(project_path, mutations)?;
    let replay = mechanical_domain_to_mutations(&exported)?;
    let fp1 = mutations_to_mechanical_snapshot(mutations)?;
    let fp2 = mutations_to_mechanical_snapshot(&replay)?;
    if fp1 != fp2 {
        return Err(FreecadHnfError::DomainParse(
            "mechanical HNF roundtrip fingerprint drift".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_snapshot() -> MechanicalSnapshot {
        MechanicalSnapshot {
            solids: vec![MechanicalSolidSnapshot {
                solid_id: "solid-bracket-1".into(),
                name: "Mounting Bracket".into(),
                material: "Aluminum".into(),
                volume_mm3: 128.5,
                step_content_hash: Some(
                    "a".repeat(64),
                ),
            }],
            constraints: vec![MechanicalConstraintSnapshot {
                constraint_id: "c-fix-1".into(),
                from_solid_id: "solid-bracket-1".into(),
                to_solid_id: None,
                constraint_type: "fixed".into(),
            }],
        }
    }

    #[test]
    fn mechanical_hnf_json_roundtrip() {
        let fp = mechanical_roundtrip_fingerprint(&sample_snapshot()).expect("roundtrip");
        assert!(fp.contains("solid-bracket-1"));
        assert!(fp.starts_with("1|1|"));
    }
}
