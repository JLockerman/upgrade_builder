use std::{
    collections::BTreeMap,
    env,
    iter::FromIterator,
    mem::{replace, swap},
    str::FromStr,
};

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
    builder.generate_scripts(|u| {
        println!("update: {} --> {}\n{}\n", u.from, u.to, u.script);
    })
}

struct UpdateBuilder {
    versions: BTreeMap<Version, String>,
}

#[derive(Debug)]
pub struct Upgrade {
    pub from: Version,
    pub to: Version,
    pub script: String,
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

    fn generate_scripts<F: FnMut(&Upgrade)>(&self, mut on_each: F) {
        let mut upgrades = vec![];
        let mut prev_version: Option<Version> = None;
        for (version, data) in &self.versions {
            let from = match &prev_version {
                Some(prev) => prev.clone(),
                None => {
                    prev_version = Some(version.clone());
                    continue;
                }
            };

            let upgrade = Upgrade {
                from,
                to: version.clone(),
                script: data.clone(),
            };

            on_each(&upgrade);

            upgrades.push(upgrade);
            prev_version = Some(version.clone());
        }

        let v = Vec::with_capacity(upgrades.len());
        let mut prev_upgrades = replace(&mut upgrades, v);
        while prev_upgrades.len() > 1 {
            for window in prev_upgrades.windows(2) {
                // println!("window {:?}", window);
                let [a, b] = match window {
                    [a, b] => [a, b],
                    _ => unreachable!(),
                };
                let update = a.merge(b);
                on_each(&update);
                upgrades.push(update);
            }
            prev_upgrades.clear();
            swap(&mut prev_upgrades, &mut upgrades);
        }
    }
}

impl Upgrade {
    fn merge(&self, other: &Self) -> Self {
        let script = String::from_iter([&*self.script, &*other.script].iter().map(|s| &**s));
        Self {
            from: self.from.clone(),
            to: other.to.clone(),
            script,
        }
    }
}
