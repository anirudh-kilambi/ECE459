use rpkg::debversion;

use crate::Packages;
use crate::packages::Dependency;

use super::RelVersionedPackageNum;

impl Packages {
    /// Computes a solution for the transitive dependencies of package_name; when there is a choice A | B | C, 
    /// chooses the first option A. Returns a Vec<i32> of package numbers.
    ///
    /// Note: does not consider which packages are installed.
        
    pub fn transitive_dep_solution(&self, package_name: &str) -> Vec<i32> {
        if !self.package_exists(package_name) {
            return vec![];
        }

        let deps: &Vec<Dependency> = &*self.dependencies.get(self.get_package_num(package_name)).unwrap();
        let mut dependency_set = vec![];

        for dep_list in deps {
            for dep in dep_list {
                let pkg_num = dep_list[0].package_num;
                if !dependency_set.contains(&pkg_num) {
                    dependency_set.push(pkg_num);
            }
            }
        }

        loop {
            let mut new_dependencies = Vec::new();
            for package_num in &dependency_set {
                let deps = &*self.dependencies.get(&package_num).unwrap();

                for dep in deps {
                    let pkg_num = dep[0].package_num;
                    if !dependency_set.contains(&pkg_num) {
                        new_dependencies.push(pkg_num);
                    }
                }
            }

            if new_dependencies.is_empty() {
                break;
            }
            dependency_set.extend(new_dependencies);
        }

        return dependency_set;
    }

    /// Computes a set of packages that need to be installed to satisfy package_name's deps given the current installed packages.
    /// When a dependency A | B | C is unsatisfied, there are two possible cases:
    ///   (1) there are no versions of A, B, or C installed; pick the alternative with the highest version number (yes, compare apples and oranges).
    ///   (2) at least one of A, B, or C is installed (say A, B), but with the wrong version; of the installed packages (A, B), pick the one with the highest version number.
    pub fn compute_how_to_install(&self, package_name: &str) -> Vec<i32> {
        if !self.package_exists(package_name) {
            return vec![];
        }
        let package_num = self.get_package_num(package_name);
        let dependencies = match self.dependencies.get(&package_num) {
            None => {return vec![*package_num]},
            Some(dependencies) => dependencies
        };

        let mut dependencies_to_add : Vec<i32> = vec![];
        // implement more sophisticated worklist
        for dependency_group in dependencies {
            if let Some(pkg_satisfying) = self.dep_is_satisfied(dependency_group) {
                continue
            }
            else {
                if let Some(package_num) = self.handle_dependency_comparison(dependency_group) {
                    if !dependencies_to_add.contains(&package_num)  {
                        dependencies_to_add.push(package_num)
                     }
                }
            }
        }
        let mut seen_deps: Vec<i32> = Vec::new();
        loop {
            let mut new_dependencies:Vec<i32> = Vec::new();
            for package_num in &dependencies_to_add {
                let deps = &*self.dependencies.get(&package_num).unwrap();
                for dep in deps {
                    if let Some(package_num) = self.handle_dependency_comparison(dep) {
                        if !&seen_deps.contains(&package_num) {
                            seen_deps.push(package_num);
                            new_dependencies.push(package_num)
                        }
                    }
                }
            }

            if new_dependencies.is_empty() {
                break;
            }
            for i in new_dependencies {
                if !dependencies_to_add.contains(&i) {
                    dependencies_to_add.push(i);
                }
            }
        }

        return dependencies_to_add;
    }

    pub fn handle_dependency_comparison(&self, dependency_group:&Vec<RelVersionedPackageNum>) -> Option<i32> {
        if let Some(pkg_satisfying) = self.dep_is_satisfied(dependency_group) {
            return None
        }
        else {
            let wrong_versions = self.dep_satisfied_by_wrong_version(dependency_group);
            if wrong_versions.len() == 0 {
                let mut versions = vec!["0".parse::<debversion::DebianVersionNum>().unwrap()];
                let mut max_package:i32 = 0;
                for package in dependency_group {
                    let package_num = package.package_num;
                    if let Some(vers) = &package.rel_version {
                        let v = vers.1.parse::<debversion::DebianVersionNum>().unwrap();
                        // compare current version with max version
                        let greater_than = debversion::cmp_debversion_with_op(&debversion::VersionRelation::StrictlyGreater, &v, &versions[versions.len() - 1]);
                        if greater_than {
                            versions.push(v);
                            max_package = package_num;
                        }

                    }
                }
                return Some(max_package)
                // check versions for all deps in the group
            }
            else {
                let min_version = "0";
                let v1 = min_version.parse::<debversion::DebianVersionNum>().unwrap();
                let mut versions = &v1;
                let mut max_package:i32 = 0;
                for package_name in wrong_versions {
                    let package_num = *self.get_package_num(package_name);
                    if let Some(v) = self.get_installed_debver(package_name) {
                        let greater_than = debversion::cmp_debversion_with_op(&debversion::VersionRelation::StrictlyGreater, &v, &versions);
                        if greater_than {
                            versions = v;
                            max_package = package_num;
                        }
                    }
                }
                return Some(max_package);
            }
        }

    }
}
