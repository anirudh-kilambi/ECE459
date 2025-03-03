use rpkg::debversion;
use crate::Packages;
use crate::packages::Dependency;

impl Packages {
    /// Gets the dependencies of package_name, and prints out whether they are satisfied (and by which library/version) or not.
    pub fn deps_available(&self, package_name: &str) {
        if !self.package_exists(package_name) {
            println!("no such package {}", package_name);
            return;
        }
        println!("Package {}:", package_name);
        let package_num = self.get_package_num(package_name);
        let deps = self.dependencies.get(package_num);
        match deps {
            None => (),
            Some(dep) => {
                for i in dep {
                    //println!("- dependency {:?}", dep);
                    let res = self.dep_is_satisfied(i);
                    match res {
                        None => {
                            println!("-> not satisfied");
                        },
                        Some(pkg) => {
                            if let Some(iv) = self.get_installed_debver(pkg) {
                                let installed_version = iv.to_string();
                                println!("+ {} satisfied by installed version {}", pkg, installed_version);
                            }
                        }
                    }
                }
            }
        }
    }

        //println!("- dependency {:?}", "dep");
        //println!("+ {} satisfied by installed version {}", "dep", "459");
        // some sort of for loop...

    /// Returns Some(package) which satisfies dependency dd, or None if not satisfied.
    pub fn dep_is_satisfied(&self, dd:&Dependency) -> Option<&str> {
        for dep in dd {
            let package_num = dep.package_num;
            let package_name = self.get_package_name(package_num);
            if let Some(iv) = self.installed_debvers.get(&package_num) {
                if let Some(vers) = &dep.rel_version {
                    let op = &vers.0;
                    let v = &vers.1.parse::<debversion::DebianVersionNum>().unwrap();
                    println!("- dependency {} \"({} {})\"", package_name, op.to_string(), &vers.1);
                    let is_satisfied = debversion::cmp_debversion_with_op(op, iv, v);
                    if is_satisfied {
                        return Some(package_name)
                    }

                }
                else {
                    if let Some(vers) = &dep.rel_version {
                        let op = &vers.0.to_string();
                        let v = &vers.1;
                        println!("- dependency {} \"({} {})\"", package_name, op, v) ;
                    }
                    else {
                        println!("- dependency {} ", package_name);
                    }
                    return Some(package_name)
                }

            }
            if let Some(vers) = &dep.rel_version {
                let op = &vers.0.to_string();
                let v = &vers.1;
                println!("- dependency {} \"({} {})\"", package_name, op, v) ;
            }
            else {
                println!("- dependency {} ", package_name);
            }
        }
        return None;
    }

    /// Returns a Vec of packages which would satisfy dependency dd but for the version.
    /// Used by the how-to-install command, which calls compute_how_to_install().
    pub fn dep_satisfied_by_wrong_version(&self, dd:&Dependency) -> Vec<&str> {
        assert! (self.dep_is_satisfied(dd).is_none());
        let mut result = vec![];
        for dep in dd {
            let package_num = dep.package_num;
            let package_name = self.get_package_name(package_num);
            if let Some(iv) = self.installed_debvers.get(&package_num) {
                if let Some(vers) = &dep.rel_version {
                    let op = &vers.0;
                    let v = &vers.1.parse::<debversion::DebianVersionNum>().unwrap();
                    let is_satisfied = debversion::cmp_debversion_with_op(op, iv, v);
                    if !is_satisfied {
                        result.push(package_name);
                    }
                }
            }
            else {
                continue
            }
        }
        return result;
    }

}
