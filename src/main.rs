use std::{collections::HashMap, env, iter::FromIterator, str::FromStr};

use semver::Version;

fn main() {
    let filename = env::args().skip(1).next().expect("need file name");
    let file = std::fs::read_to_string(filename).expect("could read file");
    let mut builder = UpdateBuilder::new();
    for line in file.lines() {
        let line: Vec<_> = line.split(':').collect();
        let version = Version::from_str(&*line[0]).unwrap();
        println!("{}:{}", version, &line[1]);
        builder.add_item(version, &line[1]);
    }
    println!("");
    println!("latest");
    builder.generate_latest_scripts(|u| {
        println!("update: {} --> {}\n{}\n", u.from, u.to, u.script);
    });
    println!("");
    println!("others");
    builder.generate_non_latest_scripts(|u| {
        println!("update: {} --> {}\n{}\n", u.from, u.to, u.script);
    });
}

struct UpdateBuilder {
    versions: HashMap<Version, String>,
}

#[derive(Debug)]
pub struct Upgrade<'a> {
    pub from: &'a Version,
    pub to: &'a Version,
    pub script: &'a str,
}

struct VersionedScript<'a> {
    pub version: &'a Version,
    pub script: &'a str,
}

impl UpdateBuilder {
    fn new() -> Self {
        Self {
            versions: Default::default(),
        }
    }

    fn add_item(&mut self, version: Version, item: &str) {
        self.versions.entry(version).or_default().push_str(item)
    }

    /// generate scripts going from any prior version to the current version
    fn generate_latest_scripts<F: FnMut(&Upgrade)>(&self, mut on_each: F) {
        // no point in upgrading if there aren't multiple versions
        if self.versions.len() <= 1 {
            return;
        }

        // sort the partial scripts by version
        let mut versions: Vec<_> = self
            .versions
            .iter()
            .map(|(version, script)| VersionedScript { version, script })
            .collect();
        versions.sort_unstable_by(|a, b| a.version.cmp(&b.version));

        // concatenate the partial scripts into a full script; all the upgrade
        // scripts will be a suffix of this one
        let full_script = String::from_iter(versions.iter().map(|v| v.script));
        let mut output = &full_script[..];

        for i in 0..versions.len() - 1 {
            // we are currently at a given version, so we should exclude things
            // installed by that version from the script; it's already there
            let current_version = &versions[i];
            let from = current_version.version;
            let installed_len = current_version.script.len();
            output = &output[installed_len..];

            let up = &Upgrade {
                from: from,
                to: versions.last().unwrap().version,
                script: &*output,
            };
            on_each(up)
        }
    }

    /// generate scripts going from every prior version to every later version,
    /// except for the latest one
    fn generate_non_latest_scripts<F: FnMut(&Upgrade)>(&self, mut on_each: F) {
        // no point in upgrading if there aren't multiple versions
        if self.versions.len() <= 1 {
            return;
        }

        // sort the partial scripts by version
        let mut versions: Vec<_> = self
            .versions
            .iter()
            .map(|(version, script)| VersionedScript { version, script })
            .collect();
        versions.sort_unstable_by(|a, b| a.version.cmp(&b.version));

        // concatenate the partial scripts into a full script; all the upgrade
        // scripts will be a substring of this one
        let full_script = String::from_iter(versions.iter().map(|v| v.script));

        let mut full_len = full_script.len();
        for j in (1..versions.len()).rev() {
            full_len -= versions[j].script.len();
            let mut output = &full_script[..full_len];

            let versions = &versions[..j];
            for i in 0..versions.len() - 1 {
                // we are currently at a given version, so we should exclude things
                // installed by that version from the script; it's already there
                let current_version = &versions[i];
                let from = current_version.version;
                let installed_len = current_version.script.len();
                output = &output[installed_len..];

                let up = &Upgrade {
                    from: from,
                    to: versions.last().unwrap().version,
                    script: &*output,
                };
                on_each(up)
            }
        }
    }
}
