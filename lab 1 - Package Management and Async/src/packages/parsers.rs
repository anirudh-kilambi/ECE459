use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use regex::Regex;

use crate::Packages;
use crate::packages::{Dependency, RelVersionedPackageNum};

use rpkg::debversion::{self, VersionRelation};

const KEYVAL_REGEX : &str = r"^(?P<key>(\w|-)+): (?P<value>.+)";
const PKGNAME_AND_VERSION_REGEX : &str = r"(?P<pkg>(\w|\.|\+|-)+)( \((?P<op>(<|=|>)(<|=|>)?) (?P<ver>.*)\))?";

impl Packages {
    /// Loads packages and version numbers from a file, calling get_package_num_inserting on the package name
    /// and inserting the appropriate value into the installed_debvers map with the parsed version number.
    pub fn parse_installed(&mut self, filename: &str) {
        let kv_regexp = Regex::new(KEYVAL_REGEX).unwrap();
        if let Ok(lines) = read_lines(filename) {
            let mut current_package_num = 0;
            for line in lines {
                if let Ok(ip) = line {
                     match kv_regexp.captures(&ip) {
                        None => (),
                        Some(caps) => {
                            let (key, value) = (caps.name("key").unwrap().as_str(), caps.name("value").unwrap().as_str());
                            current_package_num = match key {
                                "Package" => self.get_package_num_inserting(value),
                                _ => current_package_num
                            };

                            match key {
                                "Version" => {
                                    let debver = value.trim().parse::<debversion::DebianVersionNum>().unwrap();
                                    self.installed_debvers.insert(current_package_num, debver);
                                },
                                _ => ()
                            };
                        }
                }
            }
        }
        println!("Packages installed: {}", self.installed_debvers.keys().len());
    }
}

    /// Loads packages, version numbers, dependencies, and md5sums from a file, calling get_package_num_inserting on the package name
    /// and inserting the appropriate values into the dependencies, md5sum, and available_debvers maps.
    pub fn parse_packages(&mut self, filename: &str) {
        let kv_regexp = Regex::new(KEYVAL_REGEX).unwrap();
        let pkgver_regexp = Regex::new(PKGNAME_AND_VERSION_REGEX).unwrap();

        if let Ok(lines) = read_lines(filename) {
            let mut current_package_num = 0;
            for line in lines {
                if let Ok(ip) = line {
                    match kv_regexp.captures(&ip) {
                        None => (),
                        Some(cap) => {
                            let (key, value) = (cap.name("key").unwrap().as_str(), cap.name("value").unwrap().as_str());
                            current_package_num = match key {
                                "Package" => self.get_package_num_inserting(value),
                                _ => current_package_num
                            }; //current package num match
                            match key {
                                "Version" => {
                                    let debver = value.trim().parse::<debversion::DebianVersionNum>().unwrap();
                                    self.available_debvers.insert(current_package_num, debver);
                                },
                                "MD5sum" => {
                                    self.md5sums.insert(current_package_num, value.to_owned());
                                },
                                "Depends" => {
                                    let ds = self.handle_dependencies(value, &current_package_num, &pkgver_regexp);
                                    self.dependencies.insert(current_package_num, ds);
                                },
                                _ => ()
                            }; // version match
                        } //match on Some(cap)
                    }

                }
            };
        println!("Packages available: {}", self.available_debvers.keys().len());
        }
    }
    fn handle_dependencies(&mut self, depends:&str, current_package_num:&i32, pkgver_regexp:&Regex) -> Vec<Dependency>{
        let dependencies = depends.split(",");
        let mut returning :Vec<Dependency> = vec![];

        for dependency in dependencies {
            let mut test: Vec<RelVersionedPackageNum> = vec![];
            let d = dependency.trim().split("|");
            for dep in d {
                match pkgver_regexp.captures(dep) {
                    None => (),
                    Some(caps) => {
                        let pkg = caps.name("pkg").unwrap().as_str().to_string();
                        let versionrelation:Option<(VersionRelation, String)> = match caps.name("op") {
                            Some(matched) => {
                                let op = caps
                                    .name("op")
                                    .unwrap()
                                    .as_str()
                                    .parse::<debversion::VersionRelation>()
                                    .unwrap();
                                let ver = caps
                                    .name("ver")
                                    .unwrap()
                                    .as_str()
                                    .to_string();
                                Some((op, ver))
                                //let relversionpackagenum = RelVersionedPackageNum{package_num : self.get_package_num_inserting(&pkg), rel_version: versionrelation};
                            },
                            _ => None 
                        };
                        let appending:RelVersionedPackageNum = RelVersionedPackageNum{ package_num : self.get_package_num_inserting(&pkg), rel_version : versionrelation};

                        test.push(appending);

                    }
                }
            }
            returning.push(test)
        }
        returning
    }
    }



// standard template code downloaded from the Internet somewhere
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

