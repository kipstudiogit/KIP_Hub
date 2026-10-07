use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};
use url::Url;

use crate::config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreItemFileDto {
    pub filename: String,
    pub url: String,
    pub primary: bool,
    pub size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreItemDependencyDto {
    pub project_id: String,
    pub version_id: Option<String>,
    pub dependency_type: String,
    pub file_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreItemVersionDto {
    pub id: String,
    pub version_number: String,
    pub name: String,
    pub date: String,
    pub changelog: String,
    pub files: Vec<StoreItemFileDto>,
    pub dependencies: Vec<StoreItemDependencyDto>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreItemGalleryDto {
    pub url: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreItemDetailsDto {
    pub body: String,
    pub gallery: Vec<StoreItemGalleryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreItemRecordDto {
    pub project_id: String,
    pub slug: String,
    pub title: String,
    pub author: String,
    pub description: String,
    pub icon_url: String,
    pub downloads: i64,
    pub follows: i64,
    pub categories: Vec<String>,
    pub provider: String,
    pub project_type: String,
    pub is_installed: bool,
    pub installed_filename: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreSearchResultDto {
    pub success: bool,
    pub hits: Vec<StoreItemRecordDto>,
    pub total_hits: usize,
    pub msg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreDetailsResponseDto {
    pub success: bool,
    pub details: StoreItemDetailsDto,
    pub versions: Vec<StoreItemVersionDto>,
    pub msg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreInstallProgressPayload {
    pub project_id: String,
    pub filename: String,
    pub progress: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreInstallResultDto {
    pub success: bool,
    pub filename: String,
    pub installed_dependencies: Vec<String>,
    pub message: String,
}

pub struct StoreManager {
    client: reqwest::Client,
}

impl StoreManager {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("KIPStudio/KIP_Hub/1.7.0 (contact@kip.studio)"),
        );
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        Self {
            client: reqwest::Client::builder()
                .default_headers(headers)
                .redirect(reqwest::redirect::Policy::limited(10))
                .connect_timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }

    fn get_target_dir(&self, mc_dir: &Path, project_type: &str) -> PathBuf {
        match project_type {
            "resourcepack" => mc_dir.join("resourcepacks"),
            "shader" => mc_dir.join("shaderpacks"),
            "modpack" => mc_dir.join("downloads").join("modpacks"),
            _ => mc_dir.join("mods"),
        }
    }

    pub fn get_installed_filenames(&self, mc_dir: &Path, project_type: &str) -> HashSet<String> {
        let mut set = HashSet::new();
        let target_dir = self.get_target_dir(mc_dir, project_type);
        if let Ok(entries) = fs::read_dir(target_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                set.insert(name);
            }
        }
        set
    }

    fn is_item_installed(&self, installed: &HashSet<String>, slug: &str, title: &str) -> (bool, Option<String>) {
        let clean_slug = slug.trim().to_lowercase().replace('_', "-");
        let clean_title = title.trim().to_lowercase().replace(' ', "-").replace('_', "-");

        for filename in installed {
            let clean_file = filename.to_lowercase()
                .replace(".jar", "")
                .replace(".disabled", "")
                .replace(".zip", "");
            let file_stem_prefix = clean_file.split('-').next().unwrap_or(&clean_file);

            if clean_file == clean_slug
                || clean_file.starts_with(&format!("{}-", clean_slug))
                || clean_file.starts_with(&format!("{}_", clean_slug))
                || clean_file.contains(&clean_slug)
                || file_stem_prefix == clean_slug
                || clean_file.contains(&clean_title)
            {
                return (true, Some(filename.clone()));
            }
        }
        (false, None)
    }

    pub async fn search(
        &self,
        provider: &str,
        query: &str,
        project_type: &str,
        loader: &str,
        game_version: &str,
        category: &str,
        sort_index: &str,
        offset: i32,
        mc_dir_str: &str,
    ) -> Result<StoreSearchResultDto, String> {
        let mc_dir = Path::new(mc_dir_str);
        let installed = self.get_installed_filenames(mc_dir, project_type);

        if provider == "curseforge" {
            self.search_curseforge(query, project_type, loader, game_version, category, sort_index, offset, &installed).await
        } else {
            self.search_modrinth(query, project_type, loader, game_version, category, sort_index, offset, &installed).await
        }
    }

    async fn search_modrinth(
        &self,
        query: &str,
        project_type: &str,
        loader: &str,
        game_version: &str,
        category: &str,
        sort_index: &str,
        offset: i32,
        installed: &HashSet<String>,
    ) -> Result<StoreSearchResultDto, String> {
        let mut url = Url::parse("https://api.modrinth.com/v2/search").map_err(|e| e.to_string())?;

        let index_to_use = match sort_index {
            "downloads" => "downloads",
            "newest" => "newest",
            "updated" => "updated",
            "follows" => "follows",
            _ => {
                if query.trim().is_empty() {
                    "downloads"
                } else {
                    "relevance"
                }
            }
        };

        let mut facets: Vec<Vec<String>> = Vec::new();
        if !project_type.is_empty() {
            facets.push(vec![format!("project_type:{}", project_type)]);
        }
        if !loader.is_empty() && (project_type == "mod" || project_type == "modpack") {
            facets.push(vec![format!("categories:{}", loader)]);
        }
        if !game_version.is_empty() {
            facets.push(vec![format!("versions:{}", game_version)]);
        }
        if !category.is_empty() {
            facets.push(vec![format!("categories:{}", category)]);
        }

        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("limit", "16");
            pairs.append_pair("offset", &offset.to_string());
            pairs.append_pair("index", index_to_use);

            if !query.trim().is_empty() {
                pairs.append_pair("query", query.trim());
            }

            if !facets.is_empty() {
                let facets_json = serde_json::to_string(&facets).unwrap_or_default();
                pairs.append_pair("facets", &facets_json);
            }
        }

        let res = self.client.get(url).send().await.map_err(|e| e.to_string())?;
        if !res.status().is_success() {
            return Err(format!("Modrinth API error: {}", res.status()));
        }

        let data: Value = res.json().await.map_err(|e| e.to_string())?;
        let total_hits = data["total_hits"].as_u64().unwrap_or(0) as usize;
        let mut hits = Vec::new();

        if let Some(items) = data["hits"].as_array() {
            for item in items {
                let project_id = item["project_id"].as_str().unwrap_or("").to_string();
                let slug = item["slug"].as_str().unwrap_or(&project_id).to_string();
                let title = item["title"].as_str().unwrap_or("").to_string();
                let author = item["author"].as_str().unwrap_or("Unknown").to_string();
                let description = item["description"].as_str().unwrap_or("").to_string();
                let icon_url = item["icon_url"].as_str().unwrap_or("").to_string();
                let downloads = item["downloads"].as_i64().unwrap_or(0);
                let follows = item["follows"].as_i64().unwrap_or(0);

                let categories: Vec<String> = item["categories"]
                    .as_array()
                    .map(|arr| arr.iter().filter_map(|c| c.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();

                let (is_installed, installed_filename) = self.is_item_installed(installed, &slug, &title);

                hits.push(StoreItemRecordDto {
                    project_id,
                    slug,
                    title,
                    author,
                    description,
                    icon_url,
                    downloads,
                    follows,
                    categories,
                    provider: "modrinth".to_string(),
                    project_type: project_type.to_string(),
                    is_installed,
                    installed_filename,
                });
            }
        }

        Ok(StoreSearchResultDto {
            success: true,
            hits,
            total_hits,
            msg: None,
        })
    }

    async fn search_curseforge(
        &self,
        query: &str,
        project_type: &str,
        loader: &str,
        game_version: &str,
        category: &str,
        sort_index: &str,
        offset: i32,
        installed: &HashSet<String>,
    ) -> Result<StoreSearchResultDto, String> {
        let key = config::get_secret("cf_api_key");
        if key.trim().is_empty() {
            return Err("CurseForge API Key is not configured. Add it in Settings.".to_string());
        }

        let class_id = match project_type {
            "resourcepack" => 12,
            "shader" => 6552,
            "modpack" => 4471,
            _ => 6,
        };

        let sort_field = match sort_index {
            "downloads" => 2,
            "updated" | "newest" => 3,
            _ => 1,
        };

        let modloader_type = match loader {
            "forge" => 1,
            "fabric" => 4,
            "quilt" => 5,
            "neoforge" => 6,
            _ => 0,
        };

        let category_id = match category {
            "technology" => 412,
            "magic" => 406,
            "utility" => 424,
            "worldgen" => 408,
            "optimization" => 436,
            _ => 0,
        };

        let mut query_params = vec![
            ("gameId", "432".to_string()),
            ("classId", class_id.to_string()),
            ("sortField", sort_field.to_string()),
            ("sortOrder", "desc".to_string()),
            ("pageSize", "16".to_string()),
            ("index", offset.to_string()),
        ];

        if !query.is_empty() {
            query_params.push(("searchFilter", query.to_string()));
        }
        if !game_version.is_empty() {
            query_params.push(("gameVersion", game_version.to_string()));
        }
        if modloader_type != 0 && project_type == "mod" {
            query_params.push(("modLoaderType", modloader_type.to_string()));
        }
        if category_id != 0 {
            query_params.push(("categoryId", category_id.to_string()));
        }

        let query_str = query_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        let url = format!("https://api.curseforge.com/v1/mods/search?{}", query_str);

        let res = self
            .client
            .get(&url)
            .header("x-api-key", key.trim())
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(format!("CurseForge error: {}", res.status()));
        }

        let data: Value = res.json().await.map_err(|e| e.to_string())?;
        let mut hits = Vec::new();

        if let Some(items) = data["data"].as_array() {
            for item in items {
                let project_id = item["id"].as_i64().unwrap_or(0).to_string();
                let title = item["name"].as_str().unwrap_or("").to_string();
                let slug = item["slug"].as_str().unwrap_or(&title).to_string();
                let author = item["authors"]
                    .as_array()
                    .and_then(|a| a.first())
                    .and_then(|a| a["name"].as_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let icon_url = item["logo"]["thumbnailUrl"]
                    .as_str()
                    .or_else(|| item["logo"]["url"].as_str())
                    .unwrap_or("")
                    .to_string();

                let categories: Vec<String> = item["categories"]
                    .as_array()
                    .map(|arr| arr.iter().filter_map(|c| c["name"].as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();

                let (is_installed, installed_filename) = self.is_item_installed(installed, &slug, &title);

                hits.push(StoreItemRecordDto {
                    project_id,
                    slug,
                    title,
                    author,
                    description: item["summary"].as_str().unwrap_or("").to_string(),
                    icon_url,
                    downloads: item["downloadCount"].as_i64().unwrap_or(0),
                    follows: 0,
                    categories,
                    provider: "curseforge".to_string(),
                    project_type: project_type.to_string(),
                    is_installed,
                    installed_filename,
                });
            }
        }

        Ok(StoreSearchResultDto {
            success: true,
            hits,
            total_hits: 1000,
            msg: None,
        })
    }

    pub async fn get_details(
        &self,
        provider: &str,
        project_id: &str,
        loader: &str,
        game_version: &str,
    ) -> Result<StoreDetailsResponseDto, String> {
        if provider == "curseforge" {
            self.get_curseforge_details(project_id, loader, game_version).await
        } else {
            self.get_modrinth_details(project_id, loader, game_version).await
        }
    }

    async fn get_modrinth_details(
        &self,
        project_id: &str,
        loader: &str,
        game_version: &str,
    ) -> Result<StoreDetailsResponseDto, String> {
        let det_url = format!("https://api.modrinth.com/v2/project/{}", project_id);
        let det_res = self.client.get(&det_url).send().await.map_err(|e| e.to_string())?;
        if !det_res.status().is_success() {
            return Err("Project not found on Modrinth.".to_string());
        }
        let details: Value = det_res.json().await.map_err(|e| e.to_string())?;

        let mut ver_url = Url::parse(&format!("https://api.modrinth.com/v2/project/{}/version", project_id)).map_err(|e| e.to_string())?;
        {
            let mut pairs = ver_url.query_pairs_mut();
            if !loader.is_empty() {
                let l_json = serde_json::to_string(&vec![loader]).unwrap_or_default();
                pairs.append_pair("loaders", &l_json);
            }
            if !game_version.is_empty() {
                let v_json = serde_json::to_string(&vec![game_version]).unwrap_or_default();
                pairs.append_pair("game_versions", &v_json);
            }
        }

        let ver_res = self.client.get(ver_url).send().await.map_err(|e| e.to_string())?;
        let versions_raw: Value = if ver_res.status().is_success() {
            ver_res.json().await.unwrap_or(json!([]))
        } else {
            json!([])
        };

        let mut valid_versions = Vec::new();
        if let Some(arr) = versions_raw.as_array() {
            for v in arr {
                let mut files = Vec::new();
                if let Some(fls) = v["files"].as_array() {
                    for f in fls {
                        let fname = f["filename"].as_str().unwrap_or("").to_lowercase();
                        if fname.ends_with("-sources.jar") || fname.ends_with("-javadoc.jar") || fname.ends_with("-dev.jar") {
                            continue;
                        }
                        files.push(StoreItemFileDto {
                            filename: f["filename"].as_str().unwrap_or("").to_string(),
                            url: f["url"].as_str().unwrap_or("").to_string(),
                            primary: f["primary"].as_bool().unwrap_or(false),
                            size: f["size"].as_u64().unwrap_or(0) as usize,
                        });
                    }
                }

                if files.is_empty() {
                    continue;
                }

                let mut dependencies = Vec::new();
                if let Some(deps) = v["dependencies"].as_array() {
                    for d in deps {
                        dependencies.push(StoreItemDependencyDto {
                            project_id: d["project_id"].as_str().unwrap_or("").to_string(),
                            version_id: d["version_id"].as_str().map(|s| s.to_string()),
                            dependency_type: d["dependency_type"].as_str().unwrap_or("required").to_string(),
                            file_name: d["file_name"].as_str().map(|s| s.to_string()),
                        });
                    }
                }

                valid_versions.push(StoreItemVersionDto {
                    id: v["id"].as_str().unwrap_or("").to_string(),
                    version_number: v["version_number"].as_str().unwrap_or("").to_string(),
                    name: v["name"].as_str().unwrap_or("").to_string(),
                    date: v["date_published"].as_str().unwrap_or("").to_string(),
                    changelog: v["changelog"].as_str().unwrap_or("").to_string(),
                    files,
                    dependencies,
                    game_versions: v["game_versions"].as_array().map(|arr| arr.iter().filter_map(|s| s.as_str().map(|st| st.to_string())).collect()).unwrap_or_default(),
                    loaders: v["loaders"].as_array().map(|arr| arr.iter().filter_map(|s| s.as_str().map(|st| st.to_string())).collect()).unwrap_or_default(),
                });
            }
        }

        let mut gallery = Vec::new();
        if let Some(gal) = details["gallery"].as_array() {
            for g in gal {
                if let Some(url_str) = g["url"].as_str() {
                    gallery.push(StoreItemGalleryDto {
                        url: url_str.to_string(),
                        title: g["title"].as_str().map(|s| s.to_string()),
                    });
                }
            }
        }

        Ok(StoreDetailsResponseDto {
            success: true,
            details: StoreItemDetailsDto {
                body: details["body"].as_str().unwrap_or("").to_string(),
                gallery,
            },
            versions: valid_versions,
            msg: None,
        })
    }

    async fn get_curseforge_details(
        &self,
        project_id: &str,
        loader: &str,
        game_version: &str,
    ) -> Result<StoreDetailsResponseDto, String> {
        let key = config::get_secret("cf_api_key");
        if key.trim().is_empty() {
            return Err("CurseForge API Key is not configured.".to_string());
        }

        let url = format!("https://api.curseforge.com/v1/mods/{}", project_id);
        let res = self
            .client
            .get(&url)
            .header("x-api-key", key.trim())
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err("Project not found on CurseForge.".to_string());
        }

        let data: Value = res.json().await.map_err(|e| e.to_string())?;
        let mod_data = &data["data"];

        let mut files_query = Vec::new();
        if !game_version.is_empty() {
            files_query.push(format!("gameVersion={}", urlencoding::encode(game_version)));
        }
        let modloader_type = match loader {
            "forge" => 1,
            "fabric" => 4,
            "quilt" => 5,
            "neoforge" => 6,
            _ => 0,
        };
        if modloader_type != 0 {
            files_query.push(format!("modLoaderType={}", modloader_type));
        }

        let files_url = format!(
            "https://api.curseforge.com/v1/mods/{}/files?{}",
            project_id,
            files_query.join("&")
        );

        let files_res = self
            .client
            .get(&files_url)
            .header("x-api-key", key.trim())
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let files_data: Value = if files_res.status().is_success() {
            files_res.json().await.unwrap_or(json!({ "data": [] }))
        } else {
            json!({ "data": [] })
        };

        let mut valid_versions = Vec::new();
        if let Some(files) = files_data["data"].as_array() {
            for f in files {
                let id_str = f["id"].as_i64().unwrap_or(0).to_string();
                let filename = f["fileName"].as_str().unwrap_or("").to_string();
                let dl_url = f["downloadUrl"].as_str().map(|s| s.to_string()).unwrap_or_else(|| {
                    if id_str.len() >= 4 {
                        format!(
                            "https://edge.forgecdn.net/files/{}/{}/{}",
                            &id_str[..4],
                            &id_str[4..],
                            filename
                        )
                    } else {
                        String::new()
                    }
                });

                valid_versions.push(StoreItemVersionDto {
                    id: id_str,
                    version_number: f["displayName"].as_str().unwrap_or("").to_string(),
                    name: filename.clone(),
                    date: f["fileDate"].as_str().unwrap_or("").to_string(),
                    changelog: String::new(),
                    files: vec![StoreItemFileDto {
                        filename,
                        url: dl_url,
                        primary: true,
                        size: f["fileLength"].as_u64().unwrap_or(0) as usize,
                    }],
                    dependencies: Vec::new(),
                    game_versions: f["gameVersions"].as_array().map(|arr| arr.iter().filter_map(|s| s.as_str().map(|st| st.to_string())).collect()).unwrap_or_default(),
                    loaders: Vec::new(),
                });
            }
        }

        let mut gallery = Vec::new();
        if let Some(screenshots) = mod_data["screenshots"].as_array() {
            for s in screenshots {
                if let Some(url_str) = s["url"].as_str() {
                    gallery.push(StoreItemGalleryDto {
                        url: url_str.to_string(),
                        title: s["title"].as_str().map(|st| st.to_string()),
                    });
                }
            }
        }

        Ok(StoreDetailsResponseDto {
            success: true,
            details: StoreItemDetailsDto {
                body: mod_data["summary"].as_str().unwrap_or("").to_string(),
                gallery,
            },
            versions: valid_versions,
            msg: None,
        })
    }

    pub async fn download_with_progress(
        &self,
        app: &AppHandle,
        url: &str,
        dest_path: &Path,
        project_id: &str,
        filename: &str,
    ) -> Result<(), String> {
        let resp = self.client.get(url).send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("Download failed with status: {}", resp.status()));
        }

        let total_size = resp.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;
        let temp_path = dest_path.with_extension("dl_tmp");

        let mut file = File::create(&temp_path).map_err(|e| e.to_string())?;
        let mut stream = resp.bytes_stream();

        while let Some(chunk_res) = stream.next().await {
            let chunk = chunk_res.map_err(|e| e.to_string())?;
            file.write_all(&chunk).map_err(|e| e.to_string())?;
            downloaded += chunk.len() as u64;

            let progress = if total_size > 0 {
                ((downloaded as f64 / total_size as f64) * 100.0).min(100.0)
            } else {
                50.0
            };

            let _ = app.emit(
                "storeDownloadProgress",
                StoreInstallProgressPayload {
                    project_id: project_id.to_string(),
                    filename: filename.to_string(),
                    progress,
                    status: "Streaming payload...".to_string(),
                },
            );
        }

        file.sync_all().map_err(|e| e.to_string())?;
        drop(file);

        let is_valid = if let Ok(meta) = fs::metadata(&temp_path) {
            if meta.len() >= 500 {
                if let Ok(mut check_file) = File::open(&temp_path) {
                    let mut magic = [0u8; 4];
                    check_file.read_exact(&mut magic).is_ok() && &magic == &[0x50, 0x4B, 0x03, 0x04]
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if !is_valid {
            let _ = fs::remove_file(&temp_path);
            return Err("Downloaded archive failed ZIP magic integrity validation.".to_string());
        }

        if dest_path.exists() {
            let _ = fs::remove_file(dest_path);
        }

        fs::rename(&temp_path, dest_path).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn install_with_dependencies(
        &self,
        app: &AppHandle,
        provider: &str,
        project_id: &str,
        version_id: &str,
        file_url: &str,
        filename: &str,
        project_type: &str,
        mc_dir_str: &str,
        loader: &str,
        game_version: &str,
    ) -> Result<StoreInstallResultDto, String> {
        let mc_dir = Path::new(mc_dir_str);
        let target_dir = self.get_target_dir(mc_dir, project_type);
        fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;

        let clean_fname = Path::new(filename).file_name().and_then(|n| n.to_str()).unwrap_or(filename);
        let primary_dest = target_dir.join(clean_fname);
        let mut installed_dependencies = Vec::new();

        self.download_with_progress(app, file_url, &primary_dest, project_id, clean_fname).await?;

        if provider == "modrinth" && project_type == "mod" {
            let mut visited = HashSet::new();
            visited.insert(project_id.to_string());

            let mut queue = Vec::new();
            let ver_lookup = if version_id.is_empty() {
                format!("https://api.modrinth.com/v2/project/{}/version", project_id)
            } else {
                format!("https://api.modrinth.com/v2/version/{}", version_id)
            };

            if let Ok(v_resp) = self.client.get(&ver_lookup).send().await {
                if let Ok(v_data) = v_resp.json::<Value>().await {
                    let target_version = if v_data.is_array() {
                        v_data.as_array().and_then(|a| a.first()).cloned().unwrap_or(Value::Null)
                    } else {
                        v_data
                    };

                    if let Some(deps) = target_version["dependencies"].as_array() {
                        for dep in deps {
                            if dep["dependency_type"].as_str() == Some("required") {
                                if let Some(dep_proj) = dep["project_id"].as_str() {
                                    queue.push(dep_proj.to_string());
                                }
                            }
                        }
                    }
                }
            }

            let mut installed_now = self.get_installed_filenames(mc_dir, "mod");

            while let Some(dep_proj_id) = queue.pop() {
                if visited.contains(&dep_proj_id) {
                    continue;
                }
                visited.insert(dep_proj_id.clone());

                let details_res = self.get_modrinth_details(&dep_proj_id, loader, game_version).await;
                if let Ok(det) = details_res {
                    let proj_slug = det.details.body.split_whitespace().next().unwrap_or(&dep_proj_id);
                    let (already_inst, _) = self.is_item_installed(&installed_now, &dep_proj_id, proj_slug);
                    if already_inst {
                        continue;
                    }

                    if let Some(first_ver) = det.versions.first() {
                        if let Some(target_file) = first_ver.files.iter().find(|f| f.primary).or_else(|| first_ver.files.first()) {
                            let dep_clean_fname = Path::new(&target_file.filename).file_name().and_then(|n| n.to_str()).unwrap_or(&target_file.filename);
                            let dep_dest = target_dir.join(dep_clean_fname);

                            let _ = app.emit(
                                "storeDownloadProgress",
                                StoreInstallProgressPayload {
                                    project_id: dep_proj_id.clone(),
                                    filename: dep_clean_fname.to_string(),
                                    progress: 10.0,
                                    status: format!("Resolving companion {}...", dep_clean_fname),
                                },
                            );

                            if self.download_with_progress(app, &target_file.url, &dep_dest, &dep_proj_id, dep_clean_fname).await.is_ok() {
                                installed_dependencies.push(dep_clean_fname.to_string());
                                installed_now.insert(dep_clean_fname.to_lowercase());
                            }

                            for sub_dep in &first_ver.dependencies {
                                if sub_dep.dependency_type == "required" && !visited.contains(&sub_dep.project_id) {
                                    queue.push(sub_dep.project_id.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(StoreInstallResultDto {
            success: true,
            filename: clean_fname.to_string(),
            installed_dependencies,
            message: format!("Successfully installed {}", clean_fname),
        })
    }

    pub fn uninstall_item(&self, mc_dir_str: &str, project_type: &str, filename: &str) -> Result<bool, String> {
        let safe_filename = match Path::new(filename).file_name() {
            Some(f) => f.to_string_lossy().to_string(),
            None => return Err("Invalid filename.".to_string()),
        };

        let mc_dir = Path::new(mc_dir_str);
        let target_dir = self.get_target_dir(mc_dir, project_type);
        let file_path = target_dir.join(&safe_filename);
        let disabled_path = target_dir.join(format!("{}.disabled", safe_filename));

        if file_path.exists() {
            fs::remove_file(file_path).map_err(|e| e.to_string())?;
            return Ok(true);
        } else if disabled_path.exists() {
            fs::remove_file(disabled_path).map_err(|e| e.to_string())?;
            return Ok(true);
        }

        Err("File not found in active instance storage.".to_string())
    }
}