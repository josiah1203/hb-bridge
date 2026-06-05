//! HNF v0.1 save/load for KiCad layout and schematic domains.

use hnf_core::{
    parse_layout, parse_schematic, serialize_layout, serialize_schematic, validate, HnfDocument,
    HnfManifest, HnfObject, HNF_VERSION_V0_1, LAYOUT_DOMAIN, SCHEMATIC_DOMAIN,
};
use hnf_core::{
    LayoutDomain, LayoutFootprint, LayoutProperties, LayoutTrack, SchematicDomain,
    SchematicNet, SchematicProperties, SchematicSymbol, SCHEMATIC_VERSION, LAYOUT_VERSION,
};
use hnf_core::domain::HNF_TYPE_OBJECT;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sidecar_protocol::Mutation;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KicadHnfError {
    #[error("manifest validation failed: {0}")]
    Manifest(String),
    #[error("domain parse failed: {0}")]
    DomainParse(String),
    #[error("missing {domain} object in HNF document")]
    MissingDomain { domain: String },
    #[error("serialization failed: {0}")]
    Serialization(String),
    #[error("deserialization failed: {0}")]
    Deserialization(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct LayoutSnapshot {
    #[serde(default)]
    pub footprints: Vec<LayoutFootprintSnapshot>,
    #[serde(default)]
    pub tracks: Vec<LayoutTrackSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutFootprintSnapshot {
    pub refdes: String,
    pub layer: String,
    #[serde(default)]
    pub position_x: f64,
    #[serde(default)]
    pub position_y: f64,
    #[serde(default)]
    pub rotation_deg: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutTrackSnapshot {
    pub net: String,
    pub layer: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width_mm: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SchematicSnapshot {
    #[serde(default)]
    pub symbols: Vec<SchematicSymbolSnapshot>,
    #[serde(default)]
    pub nets: Vec<SchematicNetSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchematicSymbolSnapshot {
    pub refdes: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lib_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchematicNetSnapshot {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub net_class: Option<String>,
}

fn slug_id(prefix: &str, key: &str) -> String {
    let safe = key
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>();
    format!("{prefix}-{safe}")
}

pub fn layout_domain_from_snapshot(
    snapshot: &LayoutSnapshot,
    object_id: &str,
) -> Result<LayoutDomain, KicadHnfError> {
    let footprints: Vec<LayoutFootprint> = snapshot
        .footprints
        .iter()
        .enumerate()
        .map(|(_i, fp)| LayoutFootprint {
            id: slug_id("fp", &fp.refdes),
            refdes: fp.refdes.clone(),
            layer: fp.layer.clone(),
            position_x: Some(fp.position_x),
            position_y: Some(fp.position_y),
            rotation_deg: Some(fp.rotation_deg),
        })
        .collect();

    let tracks: Vec<LayoutTrack> = snapshot
        .tracks
        .iter()
        .enumerate()
        .map(|(i, tr)| LayoutTrack {
            id: slug_id("tr", &format!("{}-{}", tr.net, i)),
            net: tr.net.clone(),
            layer: tr.layer.clone(),
            width_mm: tr.width_mm,
        })
        .collect();

    if footprints.is_empty() && tracks.is_empty() {
        return Err(KicadHnfError::DomainParse(
            "layout snapshot requires at least one footprint or track".into(),
        ));
    }

    let domain = LayoutDomain {
        domain: LAYOUT_DOMAIN.into(),
        version: LAYOUT_VERSION.into(),
        hnf_type: HNF_TYPE_OBJECT.into(),
        object_id: object_id.into(),
        content_hash: None,
        refs: vec![],
        properties: LayoutProperties {
            footprints,
            tracks,
        },
    };

    parse_layout(&serialize_layout(&domain))
        .map_err(|e| KicadHnfError::DomainParse(e.to_string()))
}

pub fn layout_snapshot_from_domain(domain: &LayoutDomain) -> LayoutSnapshot {
    LayoutSnapshot {
        footprints: domain
            .properties
            .footprints
            .iter()
            .map(|fp| LayoutFootprintSnapshot {
                refdes: fp.refdes.clone(),
                layer: fp.layer.clone(),
                position_x: fp.position_x.unwrap_or(0.0),
                position_y: fp.position_y.unwrap_or(0.0),
                rotation_deg: fp.rotation_deg.unwrap_or(0.0),
            })
            .collect(),
        tracks: domain
            .properties
            .tracks
            .iter()
            .map(|tr| LayoutTrackSnapshot {
                net: tr.net.clone(),
                layer: tr.layer.clone(),
                width_mm: tr.width_mm,
            })
            .collect(),
    }
}

pub fn schematic_domain_from_snapshot(
    snapshot: &SchematicSnapshot,
    object_id: &str,
) -> Result<SchematicDomain, KicadHnfError> {
    let symbols: Vec<SchematicSymbol> = snapshot
        .symbols
        .iter()
        .map(|sym| SchematicSymbol {
            id: slug_id("sym", &sym.refdes),
            refdes: sym.refdes.clone(),
            lib_id: sym.lib_id.clone(),
            value: sym.value.clone(),
        })
        .collect();

    let nets: Vec<SchematicNet> = snapshot
        .nets
        .iter()
        .map(|net| SchematicNet {
            id: slug_id("net", &net.name),
            name: net.name.clone(),
            net_class: net.net_class.clone(),
        })
        .collect();

    if symbols.is_empty() {
        return Err(KicadHnfError::DomainParse(
            "schematic snapshot requires at least one symbol".into(),
        ));
    }

    let domain = SchematicDomain {
        domain: SCHEMATIC_DOMAIN.into(),
        version: SCHEMATIC_VERSION.into(),
        hnf_type: HNF_TYPE_OBJECT.into(),
        object_id: object_id.into(),
        content_hash: None,
        refs: vec![],
        properties: SchematicProperties {
            symbols,
            nets,
            pins: vec![],
            power_domains: vec![],
        },
    };

    parse_schematic(&serialize_schematic(&domain))
        .map_err(|e| KicadHnfError::DomainParse(e.to_string()))
}

pub fn schematic_snapshot_from_domain(domain: &SchematicDomain) -> SchematicSnapshot {
    SchematicSnapshot {
        symbols: domain
            .properties
            .symbols
            .iter()
            .map(|sym| SchematicSymbolSnapshot {
                refdes: sym.refdes.clone(),
                lib_id: sym.lib_id.clone(),
                value: sym.value.clone(),
            })
            .collect(),
        nets: domain
            .properties
            .nets
            .iter()
            .map(|net| SchematicNetSnapshot {
                name: net.name.clone(),
                net_class: net.net_class.clone(),
            })
            .collect(),
    }
}

pub fn build_layout_hnf_document(
    snapshot: &LayoutSnapshot,
    doc_id: &str,
    document_uri: &str,
) -> Result<HnfDocument, KicadHnfError> {
    let object_id = format!("{doc_id}-layout");
    let domain = layout_domain_from_snapshot(snapshot, &object_id)?;
    Ok(HnfDocument {
        manifest: HnfManifest {
            hnf_version: HNF_VERSION_V0_1.to_string(),
            doc_id: doc_id.to_string(),
            disciplines: vec![LAYOUT_DOMAIN.to_string()],
            created_at: None,
            schema_revision: None,
        },
        document_uri: document_uri.to_string(),
        metadata: json!({ "adapter": "hnf-kicad", "domain": LAYOUT_DOMAIN }),
        objects: vec![HnfObject {
            id: object_id,
            kind: LAYOUT_DOMAIN.to_string(),
            properties: serialize_layout(&domain),
        }],
    })
}

pub fn build_schematic_hnf_document(
    snapshot: &SchematicSnapshot,
    doc_id: &str,
    document_uri: &str,
) -> Result<HnfDocument, KicadHnfError> {
    let object_id = format!("{doc_id}-schematic");
    let domain = schematic_domain_from_snapshot(snapshot, &object_id)?;
    Ok(HnfDocument {
        manifest: HnfManifest {
            hnf_version: HNF_VERSION_V0_1.to_string(),
            doc_id: doc_id.to_string(),
            disciplines: vec![SCHEMATIC_DOMAIN.to_string()],
            created_at: None,
            schema_revision: None,
        },
        document_uri: document_uri.to_string(),
        metadata: json!({ "adapter": "hnf-kicad", "domain": SCHEMATIC_DOMAIN }),
        objects: vec![HnfObject {
            id: object_id,
            kind: SCHEMATIC_DOMAIN.to_string(),
            properties: serialize_schematic(&domain),
        }],
    })
}

pub fn export_hnf_json(doc: &HnfDocument) -> Result<String, KicadHnfError> {
    validate(&doc.manifest).map_err(|errs| {
        KicadHnfError::Manifest(
            errs.iter()
                .map(|e| format!("{}: {}", e.field, e.message))
                .collect::<Vec<_>>()
                .join("; "),
        )
    })?;
    serde_json::to_string_pretty(doc)
        .map_err(|e| KicadHnfError::Serialization(e.to_string()))
}

pub fn import_hnf_json(raw: &str) -> Result<HnfDocument, KicadHnfError> {
    let doc: HnfDocument =
        serde_json::from_str(raw).map_err(|e| KicadHnfError::Deserialization(e.to_string()))?;
    validate(&doc.manifest).map_err(|errs| {
        KicadHnfError::Manifest(
            errs.iter()
                .map(|e| format!("{}: {}", e.field, e.message))
                .collect::<Vec<_>>()
                .join("; "),
        )
    })?;
    Ok(doc)
}

pub fn extract_layout_domain(doc: &HnfDocument) -> Result<LayoutDomain, KicadHnfError> {
    let obj = doc
        .objects
        .iter()
        .find(|o| o.kind == LAYOUT_DOMAIN)
        .ok_or_else(|| KicadHnfError::MissingDomain {
            domain: LAYOUT_DOMAIN.into(),
        })?;
    parse_layout(&obj.properties).map_err(|e| KicadHnfError::DomainParse(e.to_string()))
}

pub fn extract_schematic_domain(doc: &HnfDocument) -> Result<SchematicDomain, KicadHnfError> {
    let obj = doc
        .objects
        .iter()
        .find(|o| o.kind == SCHEMATIC_DOMAIN)
        .ok_or_else(|| KicadHnfError::MissingDomain {
            domain: SCHEMATIC_DOMAIN.into(),
        })?;
    parse_schematic(&obj.properties).map_err(|e| KicadHnfError::DomainParse(e.to_string()))
}

pub fn layout_roundtrip_fingerprint(snapshot: &LayoutSnapshot) -> Result<String, KicadHnfError> {
    let doc = build_layout_hnf_document(snapshot, "roundtrip-doc", "hbp://roundtrip/layout")?;
    let json = export_hnf_json(&doc)?;
    let imported = import_hnf_json(&json)?;
    let domain = extract_layout_domain(&imported)?;
    let replay = layout_snapshot_from_domain(&domain);
    Ok(format!(
        "{}|{}|{}",
        replay.footprints.len(),
        replay.tracks.len(),
        replay
            .footprints
            .first()
            .map(|f| f.refdes.as_str())
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

pub fn split_mutations(mutations: &[Mutation]) -> (Vec<Mutation>, Vec<Mutation>) {
    let mut layout = Vec::new();
    let mut schematic = Vec::new();
    for m in mutations {
        if m.kind.starts_with("pcb.") {
            layout.push(m.clone());
        } else if m.kind.starts_with("schematic.") {
            schematic.push(m.clone());
        }
    }
    (layout, schematic)
}

pub fn mutations_to_layout_snapshot(mutations: &[Mutation]) -> Result<LayoutSnapshot, KicadHnfError> {
    let mut footprints = Vec::new();
    let mut tracks = Vec::new();
    for m in mutations {
        match m.kind.as_str() {
            "pcb.footprint.upsert" => {
                let refdes = m
                    .payload
                    .get("refdes")
                    .or_else(|| m.payload.get("ref"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("UNKNOWN")
                    .to_string();
                footprints.push(LayoutFootprintSnapshot {
                    refdes,
                    layer: m
                        .payload
                        .get("layer")
                        .and_then(|v| v.as_str())
                        .unwrap_or("F.Cu")
                        .to_string(),
                    position_x: m.payload.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    position_y: m.payload.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    rotation_deg: m
                        .payload
                        .get("rotation_deg")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0),
                });
            }
            "pcb.track.upsert" => {
                tracks.push(LayoutTrackSnapshot {
                    net: m
                        .payload
                        .get("net")
                        .and_then(|v| v.as_str())
                        .unwrap_or("UNKNOWN")
                        .to_string(),
                    layer: m
                        .payload
                        .get("layer")
                        .and_then(|v| v.as_str())
                        .unwrap_or("F.Cu")
                        .to_string(),
                    width_mm: m.payload.get("width_mm").and_then(|v| v.as_f64()),
                });
            }
            other => {
                return Err(KicadHnfError::DomainParse(format!(
                    "unsupported layout mutation kind: {other}"
                )));
            }
        }
    }
    Ok(LayoutSnapshot {
        footprints,
        tracks,
    })
}

pub fn mutations_to_schematic_snapshot(
    mutations: &[Mutation],
) -> Result<SchematicSnapshot, KicadHnfError> {
    let mut symbols = Vec::new();
    let mut nets = Vec::new();
    for m in mutations {
        match m.kind.as_str() {
            "schematic.symbol.upsert" | "schematic.symbol.add" | "schematic.addSymbol" => {
                let refdes = m
                    .payload
                    .get("refdes")
                    .or_else(|| m.payload.get("ref"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("UNKNOWN")
                    .to_string();
                symbols.push(SchematicSymbolSnapshot {
                    refdes,
                    lib_id: m
                        .payload
                        .get("lib_id")
                        .and_then(|v| v.as_str())
                        .map(str::to_string),
                    value: m
                        .payload
                        .get("value")
                        .and_then(|v| v.as_str())
                        .map(str::to_string),
                });
            }
            "schematic.net.upsert" | "schematic.wire.upsert" => {
                let name = m
                    .payload
                    .get("name")
                    .or_else(|| m.payload.get("net"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("UNKNOWN")
                    .to_string();
                nets.push(SchematicNetSnapshot {
                    name,
                    net_class: m
                        .payload
                        .get("net_class")
                        .and_then(|v| v.as_str())
                        .map(str::to_string),
                });
            }
            other => {
                return Err(KicadHnfError::DomainParse(format!(
                    "unsupported schematic mutation kind: {other}"
                )));
            }
        }
    }
    Ok(SchematicSnapshot { symbols, nets })
}

pub fn layout_snapshot_to_mutations(snapshot: &LayoutSnapshot) -> Vec<Mutation> {
    let mut out = Vec::new();
    for fp in &snapshot.footprints {
        out.push(Mutation {
            kind: "pcb.footprint.upsert".into(),
            payload: json!({
                "refdes": fp.refdes,
                "layer": fp.layer,
                "x": fp.position_x,
                "y": fp.position_y,
                "rotation_deg": fp.rotation_deg,
            }),
        });
    }
    for tr in &snapshot.tracks {
        out.push(Mutation {
            kind: "pcb.track.upsert".into(),
            payload: json!({
                "net": tr.net,
                "layer": tr.layer,
                "width_mm": tr.width_mm,
            }),
        });
    }
    out
}

pub fn schematic_snapshot_to_mutations(snapshot: &SchematicSnapshot) -> Vec<Mutation> {
    let mut out = Vec::new();
    for sym in &snapshot.symbols {
        out.push(Mutation {
            kind: "schematic.symbol.upsert".into(),
            payload: json!({
                "refdes": sym.refdes,
                "lib_id": sym.lib_id,
                "value": sym.value,
            }),
        });
    }
    for net in &snapshot.nets {
        out.push(Mutation {
            kind: "schematic.net.upsert".into(),
            payload: json!({
                "name": net.name,
                "net_class": net.net_class,
            }),
        });
    }
    out
}

pub fn mutations_to_layout_domain(
    project_path: &str,
    mutations: &[Mutation],
) -> Result<serde_json::Value, KicadHnfError> {
    let snap = mutations_to_layout_snapshot(mutations)?;
    let doc_id = doc_id_from_project(project_path);
    let doc = build_layout_hnf_document(&snap, &doc_id, project_path)?;
    serde_json::to_value(doc).map_err(|e| KicadHnfError::Serialization(e.to_string()))
}

pub fn mutations_to_schematic_domain(
    project_path: &str,
    mutations: &[Mutation],
) -> Result<serde_json::Value, KicadHnfError> {
    let snap = mutations_to_schematic_snapshot(mutations)?;
    let doc_id = doc_id_from_project(project_path);
    let doc = build_schematic_hnf_document(&snap, &doc_id, project_path)?;
    serde_json::to_value(doc).map_err(|e| KicadHnfError::Serialization(e.to_string()))
}

pub fn layout_domain_to_mutations(hnf_document: &serde_json::Value) -> Result<Vec<Mutation>, KicadHnfError> {
    let doc: HnfDocument = serde_json::from_value(hnf_document.clone())
        .map_err(|e| KicadHnfError::Deserialization(e.to_string()))?;
    let domain = extract_layout_domain(&doc)?;
    Ok(layout_snapshot_to_mutations(&layout_snapshot_from_domain(&domain)))
}

pub fn schematic_domain_to_mutations(
    hnf_document: &serde_json::Value,
) -> Result<Vec<Mutation>, KicadHnfError> {
    let doc: HnfDocument = serde_json::from_value(hnf_document.clone())
        .map_err(|e| KicadHnfError::Deserialization(e.to_string()))?;
    let domain = extract_schematic_domain(&doc)?;
    Ok(schematic_snapshot_to_mutations(&schematic_snapshot_from_domain(
        &domain,
    )))
}

pub fn roundtrip_layout(project_path: &str, mutations: &[Mutation]) -> Result<(), KicadHnfError> {
    let exported = mutations_to_layout_domain(project_path, mutations)?;
    let replay = layout_domain_to_mutations(&exported)?;
    let fp1 = mutations_to_layout_snapshot(mutations)?;
    let fp2 = mutations_to_layout_snapshot(&replay)?;
    if fp1 != fp2 {
        return Err(KicadHnfError::DomainParse(
            "layout HNF roundtrip fingerprint drift".into(),
        ));
    }
    Ok(())
}

pub fn roundtrip_schematic(project_path: &str, mutations: &[Mutation]) -> Result<(), KicadHnfError> {
    let exported = mutations_to_schematic_domain(project_path, mutations)?;
    let replay = schematic_domain_to_mutations(&exported)?;
    let fp1 = mutations_to_schematic_snapshot(mutations)?;
    let fp2 = mutations_to_schematic_snapshot(&replay)?;
    if fp1 != fp2 {
        return Err(KicadHnfError::DomainParse(
            "schematic HNF roundtrip fingerprint drift".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_layout_snapshot() -> LayoutSnapshot {
        LayoutSnapshot {
            footprints: vec![LayoutFootprintSnapshot {
                refdes: "R1".into(),
                layer: "F.Cu".into(),
                position_x: 1.0,
                position_y: 2.0,
                rotation_deg: 90.0,
            }],
            tracks: vec![LayoutTrackSnapshot {
                net: "GND".into(),
                layer: "F.Cu".into(),
                width_mm: Some(0.2),
            }],
        }
    }

    #[test]
    fn layout_hnf_json_roundtrip() {
        let snap = sample_layout_snapshot();
        let fp = layout_roundtrip_fingerprint(&snap).expect("roundtrip");
        assert!(fp.contains("R1"));
        assert!(fp.starts_with("1|1|"));
    }

    #[test]
    fn schematic_hnf_json_roundtrip() {
        let snap = SchematicSnapshot {
            symbols: vec![SchematicSymbolSnapshot {
                refdes: "U1".into(),
                lib_id: Some("Device:U".into()),
                value: Some("MCU".into()),
            }],
            nets: vec![SchematicNetSnapshot {
                name: "VCC".into(),
                net_class: None,
            }],
        };
        let doc =
            build_schematic_hnf_document(&snap, "sch-doc", "hbp://roundtrip/schematic").expect("build");
        let json = export_hnf_json(&doc).expect("export");
        let imported = import_hnf_json(&json).expect("import");
        let domain = extract_schematic_domain(&imported).expect("extract");
        let replay = schematic_snapshot_from_domain(&domain);
        assert_eq!(replay.symbols[0].refdes, "U1");
        assert_eq!(replay.nets[0].name, "VCC");
    }
}
