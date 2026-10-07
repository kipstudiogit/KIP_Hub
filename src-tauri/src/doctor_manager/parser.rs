use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;
use regex::Regex;
use serde_json::Value;
use zip::ZipArchive;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct BytecodeSignature {
    pub baseline_class_version: u16,
    pub total_classes: usize,
    pub is_multi_release: bool,
}

#[derive(Clone, Debug)]
pub struct ModDescriptor {
    pub filename: String,
    pub id: String,
    pub name: String,
    pub version: String,
    pub loaders: HashSet<String>,
    pub depends: HashMap<String, String>,
    pub breaks: HashMap<String, String>,
    pub provides: HashSet<String>,
    pub embedded_jij: Vec<String>,
    pub is_coremod: bool,
    pub is_corrupted: bool,
    pub bytecode: BytecodeSignature,
}

pub struct JarParser {
    re_id: Regex,
    re_name: Regex,
    re_ver: Regex,
}

impl JarParser {
    pub fn new() -> Self {
        Self {
            re_id: Regex::new(r#"modId\s*=\s*"([^"]+)""#).unwrap(),
            re_name: Regex::new(r#"displayName\s*=\s*"([^"]+)""#).unwrap(),
            re_ver: Regex::new(r#"version\s*=\s*"([^"]+)""#).unwrap(),
        }
    }

    fn sample_bytecode(&self, archive: &mut ZipArchive<File>, is_multi_release: bool) -> BytecodeSignature {
        let mut baseline_class_version = 52;
        let mut total_classes = 0;
        let mut inspected_samples = 0;

        for i in 0..archive.len() {
            if let Ok(mut zf) = archive.by_index(i) {
                let name = zf.name().to_string();
                if name.ends_with(".class") {
                    if is_multi_release && name.starts_with("META-INF/versions/") {
                        continue;
                    }
                    if name.starts_with("META-INF/") {
                        continue;
                    }
                    total_classes += 1;

                    if inspected_samples < 3 {
                        let mut header = [0u8; 8];
                        if zf.read_exact(&mut header).is_ok() && &header[0..4] == &[0xCA, 0xFE, 0xBA, 0xBE] {
                            let major = u16::from_be_bytes([header[6], header[7]]);
                            if major > baseline_class_version {
                                baseline_class_version = major;
                            }
                            inspected_samples += 1;
                        }
                    }
                }
            }
        }

        BytecodeSignature {
            baseline_class_version,
            total_classes,
            is_multi_release,
        }
    }

    fn inspect_embedded_jij(&self, zip_data: &[u8], provides: &mut HashSet<String>, jij_names: &mut Vec<String>) {
        let cursor = Cursor::new(zip_data);
        if let Ok(mut inner_zip) = ZipArchive::new(cursor) {
            if let Ok(mut f_mod) = inner_zip.by_name("fabric.mod.json") {
                let mut content = String::new();
                if f_mod.read_to_string(&mut content).is_ok() {
                    if let Ok(data) = serde_json::from_str::<Value>(&content) {
                        if let Some(id) = data["id"].as_str() {
                            let clean_id = id.to_lowercase();
                            jij_names.push(clean_id.clone());
                            provides.insert(clean_id);
                        }
                        if let Some(prov) = data["provides"].as_array() {
                            for p in prov {
                                if let Some(s) = p.as_str() {
                                    provides.insert(s.to_lowercase());
                                }
                            }
                        }
                    }
                }
            }
            if let Ok(mut q_mod) = inner_zip.by_name("quilt.mod.json") {
                let mut content = String::new();
                if q_mod.read_to_string(&mut content).is_ok() {
                    if let Ok(data) = serde_json::from_str::<Value>(&content) {
                        if let Some(id) = data["quilt_loader"]["id"].as_str() {
                            let clean_id = id.to_lowercase();
                            jij_names.push(clean_id.clone());
                            provides.insert(clean_id);
                        }
                    }
                }
            }
        }
    }

    pub fn parse(&self, path: &Path) -> ModDescriptor {
        let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let default_name = filename.replace(".jar", "").replace(".disabled", "");

        let default_bytecode = BytecodeSignature {
            baseline_class_version: 52,
            total_classes: 0,
            is_multi_release: false,
        };

        let mut descriptor = ModDescriptor {
            filename: filename.clone(),
            id: "?".to_string(),
            name: default_name,
            version: "1.0.0".to_string(),
            loaders: HashSet::new(),
            depends: HashMap::new(),
            breaks: HashMap::new(),
            provides: HashSet::new(),
            embedded_jij: Vec::new(),
            is_coremod: false,
            is_corrupted: false,
            bytecode: default_bytecode,
        };

        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => {
                descriptor.is_corrupted = true;
                return descriptor;
            }
        };

        if let Ok(meta) = file.metadata() {
            if meta.len() < 100 {
                descriptor.is_corrupted = true;
                return descriptor;
            }
        }

        let mut archive = match ZipArchive::new(file) {
            Ok(a) => a,
            Err(_) => {
                descriptor.is_corrupted = true;
                return descriptor;
            }
        };

        let manifest_content = {
            if let Ok(mut mf) = archive.by_name("META-INF/MANIFEST.MF") {
                let mut content = String::new();
                if mf.read_to_string(&mut content).is_ok() {
                    Some(content)
                } else {
                    None
                }
            } else {
                None
            }
        };

        let mut is_multi_release = false;
        if let Some(content) = manifest_content {
            if content.contains("TweakClass:") || content.contains("FMLCorePlugin:") {
                descriptor.is_coremod = true;
            }
            if content.contains("Multi-Release: true") {
                is_multi_release = true;
            }
        }

        descriptor.bytecode = self.sample_bytecode(&mut archive, is_multi_release);

        let fabric_content = {
            if let Ok(mut f_mod) = archive.by_name("fabric.mod.json") {
                let mut content = String::new();
                if f_mod.read_to_string(&mut content).is_ok() {
                    Some(content)
                } else {
                    None
                }
            } else {
                None
            }
        };

        let quilt_content = if fabric_content.is_none() {
            if let Ok(mut q_mod) = archive.by_name("quilt.mod.json") {
                let mut content = String::new();
                if q_mod.read_to_string(&mut content).is_ok() {
                    Some(content)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let neoforge_content = if fabric_content.is_none() && quilt_content.is_none() {
            if let Ok(mut t_file) = archive.by_name("META-INF/neoforge.mods.toml") {
                let mut content = String::new();
                if t_file.read_to_string(&mut content).is_ok() {
                    Some(content)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let forge_content = if fabric_content.is_none() && quilt_content.is_none() && neoforge_content.is_none() {
            if let Ok(mut t_file) = archive.by_name("META-INF/mods.toml") {
                let mut content = String::new();
                if t_file.read_to_string(&mut content).is_ok() {
                    Some(content)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        if let Some(content) = fabric_content {
            descriptor.loaders.insert("fabric".to_string());
            if let Ok(data) = serde_json::from_str::<Value>(&content) {
                if let Some(id) = data["id"].as_str() {
                    let clean = id.to_lowercase();
                    descriptor.id = clean.clone();
                    descriptor.provides.insert(clean);
                }
                if let Some(name) = data["name"].as_str() { descriptor.name = name.to_string(); }
                if let Some(ver) = data["version"].as_str() { descriptor.version = ver.to_string(); }

                if let Some(deps) = data["depends"].as_object() {
                    for (k, v) in deps {
                        let range = v.as_str().unwrap_or("*").to_string();
                        descriptor.depends.insert(k.to_lowercase(), range);
                    }
                }
                if let Some(breaks) = data["breaks"].as_object() {
                    for (k, v) in breaks {
                        let range = v.as_str().unwrap_or("*").to_string();
                        descriptor.breaks.insert(k.to_lowercase(), range);
                    }
                }
                if let Some(provides) = data["provides"].as_array() {
                    for p in provides {
                        if let Some(s) = p.as_str() {
                            descriptor.provides.insert(s.to_lowercase());
                        }
                    }
                }
            }
        } else if let Some(content) = quilt_content {
            descriptor.loaders.insert("quilt".to_string());
            if let Ok(data) = serde_json::from_str::<Value>(&content) {
                let ql = &data["quilt_loader"];
                if let Some(id) = ql["id"].as_str() {
                    let clean = id.to_lowercase();
                    descriptor.id = clean.clone();
                    descriptor.provides.insert(clean);
                }
                if let Some(ver) = ql["version"].as_str() { descriptor.version = ver.to_string(); }
                if let Some(name) = ql["metadata"]["name"].as_str() { descriptor.name = name.to_string(); }
                if let Some(deps) = ql["depends"].as_array() {
                    for dp in deps {
                        if let Some(id) = dp["id"].as_str() {
                            let range = dp["versions"].as_str().unwrap_or("*").to_string();
                            descriptor.depends.insert(id.to_lowercase(), range);
                        }
                    }
                }
                if let Some(breaks) = ql["breaks"].as_array() {
                    for br in breaks {
                        if let Some(id) = br["id"].as_str() {
                            let range = br["versions"].as_str().unwrap_or("*").to_string();
                            descriptor.breaks.insert(id.to_lowercase(), range);
                        }
                    }
                }
                if let Some(prov) = ql["provides"].as_array() {
                    for p in prov {
                        if let Some(id) = p["id"].as_str() {
                            descriptor.provides.insert(id.to_lowercase());
                        }
                    }
                }
            }
        } else if let Some(content) = neoforge_content {
            descriptor.loaders.insert("neoforge".to_string());
            if let Some(c) = self.re_id.captures(&content) {
                let id_str = c[1].to_lowercase();
                descriptor.id = id_str.clone();
                descriptor.provides.insert(id_str);
            }
            if let Some(c) = self.re_name.captures(&content) { descriptor.name = c[1].to_string(); }
            if let Some(c) = self.re_ver.captures(&content) {
                if &c[1] != "${file.jarVersion}" {
                    descriptor.version = c[1].to_string();
                }
            }
        } else if let Some(content) = forge_content {
            descriptor.loaders.insert("forge".to_string());
            if let Some(c) = self.re_id.captures(&content) {
                let id_str = c[1].to_lowercase();
                descriptor.id = id_str.clone();
                descriptor.provides.insert(id_str);
            }
            if let Some(c) = self.re_name.captures(&content) { descriptor.name = c[1].to_string(); }
            if let Some(c) = self.re_ver.captures(&content) {
                if &c[1] != "${file.jarVersion}" {
                    descriptor.version = c[1].to_string();
                }
            }
        }

        let jij_candidates: Vec<String> = archive
            .file_names()
            .filter(|n| (n.starts_with("META-INF/jars/") || n.starts_with("META-INF/jar-in-jar/")) && n.ends_with(".jar"))
            .map(|s| s.to_string())
            .collect();

        for jij_path in jij_candidates {
            if let Some(clean_file) = Path::new(&jij_path).file_name() {
                let base = clean_file.to_string_lossy().replace(".jar", "").to_lowercase();
                descriptor.embedded_jij.push(base.clone());
                descriptor.provides.insert(base);
            }

            let inner_bytes = {
                if let Ok(mut zf) = archive.by_name(&jij_path) {
                    let mut bytes = Vec::new();
                    if zf.read_to_end(&mut bytes).is_ok() {
                        Some(bytes)
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            if let Some(bytes) = inner_bytes {
                self.inspect_embedded_jij(&bytes, &mut descriptor.provides, &mut descriptor.embedded_jij);
            }
        }

        descriptor
    }
}