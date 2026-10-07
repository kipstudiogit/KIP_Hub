use std::collections::{HashMap, HashSet};
use regex::Regex;

pub struct RulesEngine {
    pub ignored_deps: HashSet<&'static str>,
    pub slug_map: HashMap<&'static str, &'static str>,
    pub fapi_submodules: HashSet<&'static str>,
    pub qsl_submodules: HashSet<&'static str>,
    re_semver: Regex,
    re_digits: Regex,
}

impl RulesEngine {
    pub fn new() -> Self {
        let mut ignored_deps = HashSet::new();
        ignored_deps.insert("java");
        ignored_deps.insert("minecraft");
        ignored_deps.insert("fabricloader");
        ignored_deps.insert("forge");
        ignored_deps.insert("neoforge");
        ignored_deps.insert("fml");
        ignored_deps.insert("quilt_loader");
        ignored_deps.insert("quilt_base");
        ignored_deps.insert("fabric");
        ignored_deps.insert("quilt_standard_libraries");

        let mut slug_map = HashMap::new();
        slug_map.insert("fabric", "fabric-api");
        slug_map.insert("fabric-api", "fabric-api");
        slug_map.insert("qsl", "qsl");
        slug_map.insert("quilt_standard_libraries", "qsl");
        slug_map.insert("cloth_config", "cloth-config");
        slug_map.insert("cloth-config2", "cloth-config");
        slug_map.insert("cloth-config", "cloth-config");
        slug_map.insert("architectury", "architectury-api");
        slug_map.insert("architectury-api", "architectury-api");
        slug_map.insert("curios", "curios-api");
        slug_map.insert("geckolib", "geckolib");
        slug_map.insert("citresewn", "cit-resewn");
        slug_map.insert("optifine", "optifabric");
        slug_map.insert("indium", "indium");
        slug_map.insert("sodium", "sodium");
        slug_map.insert("iris", "iris");
        slug_map.insert("modmenu", "modmenu");
        slug_map.insert("ferritecore", "ferrite-core");
        slug_map.insert("ferrite-core", "ferrite-core");
        slug_map.insert("entityculling", "entityculling");
        slug_map.insert("lithium", "lithium");
        slug_map.insert("krypton", "krypton");
        slug_map.insert("appleskin", "appleskin");
        slug_map.insert("jei", "jei");
        slug_map.insert("rei", "roughly-enough-items");
        slug_map.insert("roughlyenoughitems", "roughly-enough-items");
        slug_map.insert("emi", "emi");
        slug_map.insert("fabric-language-kotlin", "fabric-language-kotlin");
        slug_map.insert("forgeconfigapiport", "forge-config-api-port");
        slug_map.insert("puzzleslib", "puzzles-lib");
        slug_map.insert("balm", "balm");
        slug_map.insert("collective", "collective");

        let mut fapi_submodules = HashSet::new();
        fapi_submodules.insert("fabric-api-base");
        fapi_submodules.insert("fabric-rendering-v1");
        fapi_submodules.insert("fabric-rendering-data-attachment-v1");
        fapi_submodules.insert("fabric-rendering-fluids-v1");
        fapi_submodules.insert("fabric-resource-loader-v0");
        fapi_submodules.insert("fabric-lifecycle-events-v1");
        fapi_submodules.insert("fabric-networking-api-v1");
        fapi_submodules.insert("fabric-models-v0");
        fapi_submodules.insert("fabric-renderer-api-v1");
        fapi_submodules.insert("fabric-renderer-indigo");
        fapi_submodules.insert("fabric-screen-api-v1");
        fapi_submodules.insert("fabric-key-binding-api-v1");
        fapi_submodules.insert("fabric-content-registries-v0");
        fapi_submodules.insert("fabric-command-api-v2");
        fapi_submodules.insert("fabric-biome-api-v1");
        fapi_submodules.insert("fabric-item-api-v1");
        fapi_submodules.insert("fabric-object-builder-api-v1");
        fapi_submodules.insert("fabric-events-interaction-v0");
        fapi_submodules.insert("fabric-transitive-access-wideners-v1");
        fapi_submodules.insert("fabric-data-generation-api-v1");
        fapi_submodules.insert("fabric-convention-tags-v1");
        fapi_submodules.insert("fabric-convention-tags-v2");
        fapi_submodules.insert("fabric-sound-api-v1");
        fapi_submodules.insert("fabric-block-view-api-v2");
        fapi_submodules.insert("fabric-loot-api-v2");

        let mut qsl_submodules = HashSet::new();
        qsl_submodules.insert("qsl_base");
        qsl_submodules.insert("quilt_base");
        qsl_submodules.insert("quilt_resource_loader");
        qsl_submodules.insert("quilt_lifecycle_events");
        qsl_submodules.insert("quilt_networking");

        Self {
            ignored_deps,
            slug_map,
            fapi_submodules,
            qsl_submodules,
            re_semver: Regex::new(r"(\d+\.\d+(?:\.\d+)?)").unwrap(),
            re_digits: Regex::new(r"\d+").unwrap(),
        }
    }

    pub fn resolve_slug(&self, raw_id: &str) -> String {
        let clean = raw_id.to_lowercase();
        self.slug_map
            .get(clean.as_str())
            .copied()
            .unwrap_or(clean.as_str())
            .to_string()
    }

    pub fn parse_semver(&self, version: &str) -> (u32, u32, u32) {
        if let Some(caps) = self.re_semver.captures(version) {
            if let Some(matched) = caps.get(1) {
                let parts: Vec<u32> = matched
                    .as_str()
                    .split('.')
                    .filter_map(|s| s.parse::<u32>().ok())
                    .collect();
                return (
                    parts.get(0).copied().unwrap_or(0),
                    parts.get(1).copied().unwrap_or(0),
                    parts.get(2).copied().unwrap_or(0),
                );
            }
        }
        let digits: Vec<u32> = self
            .re_digits
            .find_iter(version)
            .filter_map(|m| m.as_str().parse::<u32>().ok())
            .collect();
        (
            digits.get(0).copied().unwrap_or(0),
            digits.get(1).copied().unwrap_or(0),
            digits.get(2).copied().unwrap_or(0),
        )
    }

    fn matches_single_constraint(&self, curr: (u32, u32, u32), constraint: &str) -> bool {
        let clean = constraint.trim();
        if clean.is_empty() || clean == "*" {
            return true;
        }

        if clean.starts_with(">=") {
            let req = self.parse_semver(clean.trim_start_matches(">=").trim());
            return curr >= req;
        }
        if clean.starts_with('>') {
            let req = self.parse_semver(clean.trim_start_matches('>').trim());
            return curr > req;
        }
        if clean.starts_with("<=") {
            let req = self.parse_semver(clean.trim_start_matches("<=").trim());
            return curr <= req;
        }
        if clean.starts_with('<') {
            let req = self.parse_semver(clean.trim_start_matches('<').trim());
            return curr < req;
        }
        if clean.starts_with('~') {
            let req = self.parse_semver(clean.trim_start_matches('~').trim());
            return curr.0 == req.0 && curr.1 == req.1 && curr >= req;
        }
        if clean.starts_with('^') {
            let req = self.parse_semver(clean.trim_start_matches('^').trim());
            return curr.0 == req.0 && curr.1 == req.1 && curr >= req;
        }
        if clean.ends_with(".*") || clean.ends_with(".x") {
            let base = clean.trim_end_matches(".*").trim_end_matches(".x");
            let req = self.parse_semver(base);
            return curr.0 == req.0 && curr.1 == req.1;
        }

        let req = self.parse_semver(clean.trim_start_matches('=').trim());
        curr == req
    }

    pub fn matches_semver_range(&self, current: &str, range_expr: &str) -> bool {
        let clean_range = range_expr.trim();
        if clean_range.is_empty() || clean_range == "*" {
            return true;
        }

        let curr = self.parse_semver(current);

        let or_branches: Vec<&str> = clean_range.split("||").collect();
        for branch in or_branches {
            let and_parts: Vec<&str> = branch.split_whitespace().collect();
            let mut branch_valid = true;
            for part in and_parts {
                let clean_part = part.trim().trim_matches(',');
                if !self.matches_single_constraint(curr, clean_part) {
                    branch_valid = false;
                    break;
                }
            }
            if branch_valid {
                return true;
            }
        }
        false
    }

    pub fn matches_minecraft_version(&self, current_mc: &str, req_expr: &str) -> bool {
        let clean_expr = req_expr.trim();
        if clean_expr.is_empty() || clean_expr == "*" {
            return true;
        }

        let curr = self.parse_semver(current_mc);
        let is_curr_calver = curr.0 >= 25;

        let contains_calver_req = clean_expr.split(|c: char| c.is_whitespace() || c == ',' || c == '|' || c == '<' || c == '>' || c == '=')
            .filter(|s| !s.is_empty())
            .any(|token| {
                let p = self.parse_semver(token);
                p.0 >= 25
            });

        let contains_classic_req = clean_expr.split(|c: char| c.is_whitespace() || c == ',' || c == '|' || c == '<' || c == '>' || c == '=')
            .filter(|s| !s.is_empty())
            .any(|token| {
                let p = self.parse_semver(token);
                p.0 == 1
            });

        if is_curr_calver && contains_classic_req && !contains_calver_req {
            return false;
        }

        if !is_curr_calver && contains_calver_req && !contains_classic_req {
            return false;
        }

        if is_curr_calver {
            let or_branches: Vec<&str> = clean_expr.split("||").collect();
            for branch in or_branches {
                let and_parts: Vec<&str> = branch.split_whitespace().collect();
                let mut branch_valid = true;
                for part in and_parts {
                    let clean_part = part.trim().trim_matches(',');
                    let req = self.parse_semver(clean_part);
                    if req.0 >= 25 {
                        if clean_part.starts_with(">=") {
                            if !(curr.0 == req.0 && curr.1 >= req.1) { branch_valid = false; break; }
                        } else if clean_part.starts_with("<=") {
                            if !(curr.0 == req.0 && curr.1 <= req.1) { branch_valid = false; break; }
                        } else if clean_part.starts_with('<') {
                            if !(curr.0 < req.0 || (curr.0 == req.0 && curr.1 < req.1)) { branch_valid = false; break; }
                        } else if clean_part.starts_with('>') {
                            if !(curr.0 > req.0 || (curr.0 == req.0 && curr.1 > req.1)) { branch_valid = false; break; }
                        } else {
                            if !(curr.0 == req.0 && curr.1 == req.1) { branch_valid = false; break; }
                        }
                    }
                }
                if branch_valid {
                    return true;
                }
            }
            return false;
        }

        self.matches_semver_range(current_mc, clean_expr)
    }

    pub fn is_internal_module(&self, dep_id: &str) -> bool {
        if dep_id.starts_with("fabric-") && dep_id != "fabric-api" {
            return true;
        }
        if dep_id.starts_with("quilt_") && dep_id != "quilt_loader" {
            return true;
        }
        self.ignored_deps.contains(dep_id)
            || self.fapi_submodules.contains(dep_id)
            || self.qsl_submodules.contains(dep_id)
    }
}