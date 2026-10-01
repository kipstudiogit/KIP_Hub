use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::collections::{HashMap, HashSet};
use zip::ZipArchive;
use regex::Regex;
use serde_json::{json, Value};
use crate::api_manager::ApiManager;

#[derive(Clone)]
struct ModInfo {
    filename: String,
    id: String,
    name: String,
    version: String,
    loaders: Vec<String>,
    depends: Vec<String>,
    breaks: Vec<String>,
    mixin_classes: Vec<String>,
    is_coremod: bool,
    jij: Vec<String>,
    is_corrupted: bool,
}

pub struct DoctorManager {
    ignored_deps: HashSet<&'static str>,
    dependency_map: HashMap<&'static str, &'static str>,
    re_semver: Regex,
    re_digits: Regex,
    re_mc_version: Regex,
    re_id: Regex,
    re_name: Regex,
    re_ver: Regex,
    re_cfg_ext: Regex,
}

impl DoctorManager {
    pub fn new() -> Self {
        let mut ignored_deps = HashSet::new();
        ignored_deps.insert("fabricloader");
        ignored_deps.insert("java");
        ignored_deps.insert("minecraft");
        ignored_deps.insert("forge");
        ignored_deps.insert("neoforge");
        ignored_deps.insert("fml");
        ignored_deps.insert("quilt_loader");
        ignored_deps.insert("quilt_base");
        ignored_deps.insert("fabric");
        ignored_deps.insert("quilt_standard_libraries");

        let mut dependency_map = HashMap::new();
        dependency_map.insert("fabric", "fabric-api");
        dependency_map.insert("fabric-api", "fabric-api");
        dependency_map.insert("qsl", "qsl");
        dependency_map.insert("quilt_standard_libraries", "qsl");

        Self {
            ignored_deps,
            dependency_map,
            re_semver: Regex::new(r"(\d+\.\d+(?:\.\d+)?)").unwrap(),
            re_digits: Regex::new(r"\d+").unwrap(),
            re_mc_version: Regex::new(r"(1\.\d{2}(?:\.\d+)?)").unwrap(),
            re_id: Regex::new(r#"modId\s*=\s*"([^"]+)""#).unwrap(),
            re_name: Regex::new(r#"displayName\s*=\s*"([^"]+)""#).unwrap(),
            re_ver: Regex::new(r#"version\s*=\s*"([^"]+)""#).unwrap(),
            re_cfg_ext: Regex::new(r"\.(json|toml|json5)$").unwrap(),
        }
    }

    fn parse_semver(&self, version: &str) -> Vec<u32> {
        if let Some(caps) = self.re_semver.captures(version) {
            if let Some(matched) = caps.get(1) {
                return matched.as_str().split('.')
                    .filter_map(|s| s.parse::<u32>().ok())
                    .collect();
            }
        }
        self.re_digits.find_iter(version)
            .filter_map(|m| m.as_str().parse::<u32>().ok())
            .collect()
    }

    fn extract_mc_version(&self, filename: &str) -> Option<String> {
        self.re_mc_version.captures(filename).and_then(|c| c.get(1)).map(|m| m.as_str().to_string())
    }

    fn is_internal_module(&self, dep_id: &str) -> bool {
        if dep_id.starts_with("fabric-") && dep_id != "fabric-api" {
            return true;
        }
        if dep_id.starts_with("quilt_") && dep_id != "quilt_loader" {
            return true;
        }
        self.ignored_deps.contains(dep_id)
    }

    fn parse_jar(&self, path: &Path) -> ModInfo {
        let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let default_name = filename.replace(".jar", "").replace(".disabled", "");
        
        let mut info = ModInfo {
            filename: filename.clone(),
            id: "?".to_string(),
            name: default_name,
            version: "?".to_string(),
            loaders: Vec::new(),
            depends: Vec::new(),
            breaks: Vec::new(),
            mixin_classes: Vec::new(),
            is_coremod: false,
            jij: Vec::new(),
            is_corrupted: false,
        };

        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => {
                info.is_corrupted = true;
                return info;
            }
        };

        let mut archive = match ZipArchive::new(file) {
            Ok(a) => a,
            Err(_) => {
                info.is_corrupted = true;
                return info;
            }
        };

        if let Ok(mut mf) = archive.by_name("META-INF/MANIFEST.MF") {
            let mut content = String::new();
            if mf.read_to_string(&mut content).is_ok() {
                if content.contains("TweakClass:") || content.contains("FMLCorePlugin:") {
                    info.is_coremod = true;
                }
            }
        }

        let mut is_fabric = false;
        if let Ok(mut f_mod) = archive.by_name("fabric.mod.json") {
            info.loaders.push("fabric".to_string());
            is_fabric = true;
            let mut content = String::new();
            if f_mod.read_to_string(&mut content).is_ok() {
                if let Ok(data) = serde_json::from_str::<Value>(&content) {
                    if let Some(id) = data["id"].as_str() { info.id = id.to_string(); }
                    if let Some(name) = data["name"].as_str() { info.name = name.to_string(); }
                    if let Some(ver) = data["version"].as_str() { info.version = ver.to_string(); }
                    if let Some(deps) = data["depends"].as_object() {
                        info.depends.extend(deps.keys().cloned());
                    }
                    if let Some(breaks) = data["breaks"].as_object() {
                        info.breaks.extend(breaks.keys().cloned());
                    }
                    if let Some(jars) = data["jars"].as_array() {
                        for j in jars {
                            if let Some(fp) = j["file"].as_str() {
                                info.jij.push(fp.to_string());
                            }
                        }
                    }
                }
            }
        }

        if !is_fabric {
            let mut is_quilt = false;
            if let Ok(mut q_mod) = archive.by_name("quilt.mod.json") {
                info.loaders.push("quilt".to_string());
                is_quilt = true;
                let mut content = String::new();
                if q_mod.read_to_string(&mut content).is_ok() {
                    if let Ok(data) = serde_json::from_str::<Value>(&content) {
                        let ql = &data["quilt_loader"];
                        if let Some(id) = ql["id"].as_str() { info.id = id.to_string(); }
                        if let Some(ver) = ql["version"].as_str() { info.version = ver.to_string(); }
                        if let Some(name) = ql["metadata"]["name"].as_str() { info.name = name.to_string(); }
                        if let Some(deps) = ql["depends"].as_array() {
                            for dp in deps {
                                if let Some(id) = dp["id"].as_str() {
                                    info.depends.push(id.to_string());
                                }
                            }
                        }
                    }
                }
            }

            if !is_quilt {
                let targets = ["META-INF/neoforge.mods.toml", "META-INF/mods.toml"];
                for target in targets {
                    let mut found = false;
                    if let Ok(mut t_file) = archive.by_name(target) {
                        info.loaders.push(if target.contains("neoforge") { "neoforge".to_string() } else { "forge".to_string() });
                        found = true;
                        let mut content = String::new();
                        if t_file.read_to_string(&mut content).is_ok() {
                            if let Some(c) = self.re_id.captures(&content) { info.id = c[1].to_string(); }
                            if let Some(c) = self.re_name.captures(&content) { info.name = c[1].to_string(); }
                            if let Some(c) = self.re_ver.captures(&content) {
                                if &c[1] != "${file.jarVersion}" {
                                    info.version = c[1].to_string();
                                }
                            }
                        }
                    }
                    if found {
                        break;
                    }
                }
            }
        }

        info
    }

    pub fn run_analysis(&self, mods_dir: &str, config_dir: &str) -> Value {
        let m_dir = Path::new(mods_dir);
        if !m_dir.exists() {
            return json!({
                "is_clean": false,
                "issues": [{
                    "id": "no_folder",
                    "type": "CRITICAL",
                    "text": "The 'mods' folder was not found in your instance.",
                    "action": "NONE",
                    "target": ""
                }]
            });
        }

        let mut issues = Vec::new();
        let mut ids_present: HashMap<String, String> = HashMap::new();
        let mut deps: HashMap<String, (String, Vec<String>)> = HashMap::new();
        let mut conflicts: HashMap<String, (String, Vec<String>)> = HashMap::new();
        let mut jij_provided: HashSet<String> = HashSet::new();
        let mut version_freq: HashMap<String, usize> = HashMap::new();
        let mut file_to_version: HashMap<String, String> = HashMap::new();
        let mut file_to_loaders: HashMap<String, Vec<String>> = HashMap::new();

        let mut optifine_present = false;
        let mut sodium_present = false;
        let mut optifabric_present = false;
        let mut optifine_file = String::new();
        let mut has_coremods = false;

        let entries = match fs::read_dir(m_dir) {
            Ok(e) => e,
            Err(e) => return json!({ "is_clean": false, "issues": [{"id": "fs_err", "type": "CRITICAL", "text": e.to_string(), "action": "NONE", "target": ""}] }),
        };

        for entry in entries.filter_map(|e| e.ok()) {
            let fp = entry.path();
            if !fp.is_file() { continue; }
            let f = entry.file_name().to_string_lossy().to_string();
            if f.ends_with(".disabled") { continue; }

            if !f.ends_with(".jar") {
                let fl = f.to_lowercase();
                if fl.ends_with(".zip") || fl.ends_with(".rar") || fl.ends_with(".txt") || fl.ends_with(".json") {
                    issues.push(json!({
                        "id": format!("inv_{}", f),
                        "type": "WARNING",
                        "text": format!("Found an invalid file '{}'. Only .jar files belong in the mods folder.", f),
                        "action": "DELETE",
                        "target": f
                    }));
                }
                continue;
            }

            let m = self.parse_jar(&fp);

            if m.is_corrupted {
                issues.push(json!({
                    "id": format!("corr_{}", f),
                    "type": "CRITICAL",
                    "text": format!("The file '{}' is corrupted, empty, or not a valid Java archive.", f),
                    "action": "DELETE",
                    "target": f
                }));
                continue;
            }

            if m.id == "?" && !m.is_coremod {
                issues.push(json!({
                    "id": format!("unk_{}", f),
                    "type": "WARNING",
                    "text": format!("'{}' doesn't seem to be a standard mod. It might cause crashes.", f),
                    "action": "DELETE",
                    "target": f
                }));
                continue;
            }

            if !m.loaders.is_empty() {
                file_to_loaders.insert(f.clone(), m.loaders.clone());
            }

            if let Some(ext_ver) = self.extract_mc_version(&f) {
                *version_freq.entry(ext_ver.clone()).or_insert(0) += 1;
                file_to_version.insert(f.clone(), ext_ver);
            }

            if m.id != "?" {
                if let Some(old_f) = ids_present.get(&m.id) {
                    let old_fp = m_dir.join(old_f);
                    let old_m = self.parse_jar(&old_fp);

                    let v1 = self.parse_semver(&m.version);
                    let v2 = self.parse_semver(&old_m.version);

                    if v1 > v2 {
                        issues.push(json!({
                            "id": format!("dup_{}_{}", m.id, old_f),
                            "type": "WARNING",
                            "text": format!("Found a duplicate of '{}'. Keeping the newer version '{}' and removing '{}'.", m.name, f, old_f),
                            "action": "DELETE",
                            "target": old_f.clone()
                        }));
                        ids_present.insert(m.id.clone(), f.clone());
                    } else {
                        issues.push(json!({
                            "id": format!("dup_{}_{}", m.id, f),
                            "type": "WARNING",
                            "text": format!("Found a duplicate of '{}'. This file ('{}') is older or identical to '{}'.", m.name, f, old_f),
                            "action": "DELETE",
                            "target": f.clone()
                        }));
                    }
                } else {
                    ids_present.insert(m.id.clone(), f.clone());
                }

                if !m.depends.is_empty() {
                    deps.insert(m.id.clone(), (m.name.clone(), m.depends.clone()));
                }
                if !m.breaks.is_empty() {
                    conflicts.insert(m.id.clone(), (m.name.clone(), m.breaks.clone()));
                }
            }

            if m.is_coremod {
                has_coremods = true;
            }

            for j in m.jij {
                let j_clean = j.replace(".jar", "");
                let j_name = Path::new(&j_clean).file_name().unwrap_or_default().to_string_lossy().to_string();
                jij_provided.insert(j_name.clone());
                if let Some(first) = j_name.split('-').next() {
                    jij_provided.insert(first.to_string());
                }
            }

            let fl = f.to_lowercase();
            if fl.contains("optifine") {
                optifine_present = true;
                optifine_file = f.clone();
            }
            if fl.contains("optifabric") {
                optifabric_present = true;
            }
            if fl.contains("sodium") || fl.contains("rubidium") || fl.contains("embeddium") {
                sodium_present = true;
            }
        }

        let dominant_mc_version = version_freq.into_iter().max_by_key(|&(_, count)| count).map(|(v, _)| v);
        
        let mut loaders_count: HashMap<String, usize> = HashMap::new();
        for lds in file_to_loaders.values() {
            for ld in lds {
                *loaders_count.entry(ld.clone()).or_insert(0) += 1;
            }
        }
        let dominant_loader = loaders_count.into_iter().max_by_key(|&(_, count)| count).map(|(l, _)| l).unwrap_or_else(|| "fabric".to_string());

        for (f, lds) in &file_to_loaders {
            if dominant_loader == "fabric" || dominant_loader == "quilt" {
                if (lds.contains(&"forge".to_string()) || lds.contains(&"neoforge".to_string())) && !lds.contains(&"fabric".to_string()) && !lds.contains(&"quilt".to_string()) {
                    issues.push(json!({
                        "id": format!("wrong_loader_{}", f),
                        "type": "CRITICAL",
                        "text": format!("'{}' is a Forge mod, which cannot run on your Fabric/Quilt setup. It must be removed.", f),
                        "action": "DELETE",
                        "target": f
                    }));
                }
            }
        }

        if let Some(target_ver) = dominant_mc_version {
            for (f, v) in file_to_version {
                if v != target_ver && !v.contains(&target_ver) && !target_ver.contains(&v) {
                    issues.push(json!({
                        "id": format!("ver_{}", f),
                        "type": "CRITICAL",
                        "text": format!("Version mismatch! '{}' is built for Minecraft {}, but your modpack is running {}.", f, v, target_ver),
                        "action": "DELETE",
                        "target": f
                    }));
                }
            }
        }

        if optifine_present {
            if dominant_loader == "fabric" && !optifabric_present {
                issues.push(json!({
                    "id": "optifabric_missing",
                    "type": "CRITICAL",
                    "text": "OptiFine on Fabric will crash without the OptiFabric bridge.",
                    "action": "DOWNLOAD",
                    "target": "optifabric"
                }));
            }
            if sodium_present {
                issues.push(json!({
                    "id": "optifine_conflict",
                    "type": "CRITICAL",
                    "text": "OptiFine is fundamentally incompatible with Sodium/Embeddium. You must choose one.",
                    "action": "DELETE",
                    "target": optifine_file
                }));
            }
        }

        if has_coremods && (dominant_loader == "fabric" || dominant_loader == "quilt") {
            issues.push(json!({
                "id": "coremod_warning",
                "type": "WARNING",
                "text": "Legacy CoreMods detected on a modern loader. This often leads to severe instability.",
                "action": "NONE",
                "target": ""
            }));
        }

        for (mid, (mname, req_deps)) in deps {
            for did in req_deps {
                if self.is_internal_module(&did) {
                    continue;
                }
                let mapped = self.dependency_map.get(did.as_str()).copied().unwrap_or(did.as_str());
                if !ids_present.contains_key(mapped) && !jij_provided.contains(mapped) {
                    issues.push(json!({
                        "id": format!("dep_{}_{}", mid, did),
                        "type": "WARNING",
                        "text": format!("The mod '{}' requires '{}' to work properly.", mname, mapped),
                        "action": "DOWNLOAD",
                        "target": mapped
                    }));
                }
            }
        }

        for (mid, (mname, breaks)) in conflicts {
            for cid in breaks {
                if let Some(target_f) = ids_present.get(&cid) {
                    issues.push(json!({
                        "id": format!("conf_{}_{}", mid, cid),
                        "type": "CRITICAL",
                        "text": format!("'{}' explicitly marks '{}' as incompatible! One must be removed.", mname, cid),
                        "action": "DELETE",
                        "target": target_f
                    }));
                }
            }
        }

        let cfg_dir = Path::new(config_dir);
        if cfg_dir.exists() {
            if let Ok(c_entries) = fs::read_dir(cfg_dir) {
                let ignored_cfgs = ["forge", "fabric", "quilt", "neoforge", "minecraft"];
                for c_entry in c_entries.filter_map(|e| e.ok()) {
                    let cf_name = c_entry.file_name().to_string_lossy().to_string();
                    if cf_name.ends_with(".json") || cf_name.ends_with(".toml") || cf_name.ends_with(".json5") {
                        let base = self.re_cfg_ext.replace(&cf_name, "").to_string();
                        let cfg_id = base.split('-').next().unwrap_or("");
                        if !ids_present.contains_key(cfg_id) && !ignored_cfgs.contains(&cfg_id) && !jij_provided.contains(cfg_id) {
                            issues.push(json!({
                                "id": format!("cfg_{}", cf_name),
                                "type": "WARNING",
                                "text": format!("Found an orphaned config file for a removed mod: '{}'. It is safe to delete.", cf_name),
                                "action": "DELETE",
                                "target": format!("../config/{}", cf_name)
                            }));
                        }
                    }
                }
            }
        }

        json!({
            "is_clean": issues.is_empty(),
            "issues": issues
        })
    }

    pub async fn apply_fixes(&self, mods_dir: &str, api_manager: &ApiManager, issues: &[Value], mc_version: &str, loader: &str) -> Value {
        let m_dir = Path::new(mods_dir);
        let mut deleted = 0;
        let mut downloaded = 0;
        let mut errors = Vec::new();

        for issue in issues {
            let action = issue["action"].as_str().unwrap_or("");
            let target = issue["target"].as_str().unwrap_or("");

            if action == "DELETE" && !target.is_empty() {
                let target_path = if target.starts_with("../") {
                    m_dir.join(target)
                } else {
                    m_dir.join(target)
                };

                if target_path.exists() {
                    match fs::remove_file(&target_path) {
                        Ok(_) => deleted += 1,
                        Err(e) => errors.push(format!("Failed to delete '{}': {}", target, e)),
                    }
                }
            } else if action == "DOWNLOAD" && !target.is_empty() {
                match api_manager.search_modrinth(target, "mod", loader, mc_version, "", "relevance", 0).await {
                    Ok(search_res) => {
                        if let Some(hits) = search_res["hits"].as_array() {
                            if let Some(first) = hits.first() {
                                if let Some(proj_id) = first["project_id"].as_str() {
                                    let url = format!("https://api.modrinth.com/v2/project/{}/version", proj_id);
                                    let client = reqwest::Client::new();
                                    if let Ok(res) = client.get(&url).header("User-Agent", "KIPStudio/KIP_Hub").send().await {
                                        if let Ok(versions) = res.json::<Value>().await {
                                            if let Some(v_arr) = versions.as_array() {
                                                let mut target_file_url = None;
                                                let mut target_filename = None;

                                                for v in v_arr {
                                                    let matches_loader = loader.is_empty() || v["loaders"].as_array().map_or(false, |l| l.iter().any(|val| val == loader));
                                                    let matches_mc = mc_version.is_empty() || v["game_versions"].as_array().map_or(false, |g| g.iter().any(|val| val == mc_version));

                                                    if matches_loader && matches_mc {
                                                        if let Some(files) = v["files"].as_array() {
                                                            if let Some(f) = files.first() {
                                                                target_file_url = f["url"].as_str().map(|s| s.to_string());
                                                                target_filename = f["filename"].as_str().map(|s| s.to_string());
                                                                break;
                                                            }
                                                        }
                                                    }
                                                }

                                                if let (Some(dl_url), Some(fname)) = (target_file_url, target_filename) {
                                                    let dest = m_dir.join(&fname);
                                                    if let Ok(dl_res) = client.get(&dl_url).send().await {
                                                        if let Ok(bytes) = dl_res.bytes().await {
                                                            if let Ok(mut out) = File::create(&dest) {
                                                                if out.write_all(&bytes).is_ok() {
                                                                    downloaded += 1;
                                                                    continue;
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        errors.push(format!("Could not auto-download dependency '{}'.", target));
                    }
                    Err(e) => errors.push(format!("Search failed for '{}': {}", target, e)),
                }
            }
        }

        json!({
            "success": errors.is_empty(),
            "deleted": deleted,
            "downloaded": downloaded,
            "errors": errors
        })
    }
}