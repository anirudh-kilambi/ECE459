use urlencoding::encode;

use curl::easy::{Easy2, Handler, WriteError};
use curl::multi::{Easy2Handle, Multi};
use std::collections::HashMap;
use std::time::Duration;
use std::str;
use std::sync::atomic::{AtomicI32, Ordering};

use crate::Packages;

struct Collector(Box<String>);
impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        (*self.0).push_str(str::from_utf8(&data.to_vec()).unwrap());
        Ok(data.len())
    }
}

const DEFAULT_SERVER : &str = "ece459.patricklam.ca:4590";
impl Drop for Packages {
    fn drop(&mut self) {
        self.execute()
    }
}

static EASYKEY_COUNTER: AtomicI32 = AtomicI32::new(0);

pub struct AsyncState {
    server : String,
    key_pkg : HashMap<i32, (String, i32, String)>,
    url_key : HashMap<String, i32>
}

impl AsyncState {
    pub fn new() -> AsyncState {
        AsyncState {
            server : String::from(DEFAULT_SERVER),
            key_pkg : HashMap::new(),
            url_key : HashMap::new()
        }
    }
}

impl Packages {
    pub fn set_server(&mut self, new_server:&str) {
        self.async_state.server = String::from(new_server);
    }

    /// Retrieves the version number of pkg and calls enq_verify_with_version with that version number.
    pub fn enq_verify(&mut self, pkg:&str) {
        let version = self.get_available_debver(pkg);
        match version {
            None => { println!("Error: package {} not defined.", pkg); return },
            Some(v) => { 
                let vs = &v.to_string();
                self.enq_verify_with_version(pkg, vs); 
            }
        };
    }

    /// Enqueues a request for the provided version/package information. Stores any needed state to async_state so that execute() can handle the results and print out needed output.
    pub fn enq_verify_with_version(&mut self, pkg:&str, version:&str) {
        let encoded_version = encode(version);
        let url = format!("http://{}/rest/v1/checksums/{}/{}", self.async_state.server, pkg, encoded_version);
        let easykey = EASYKEY_COUNTER.fetch_add(1, Ordering::SeqCst);
        let pkg_num = self.get_package_num(pkg);
        println!("queueing request {}", url);
        self.async_state.key_pkg.insert(easykey, (String::from(pkg), *pkg_num, version.to_string()));
        self.async_state.url_key.insert(url, easykey);
    }


    /// Asks curl to perform all enqueued requests. For requests that succeed with response code 200, compares received MD5sum with local MD5sum (perhaps stored earlier). For requests that fail with 400+, prints error message.
    pub fn execute(&mut self) {
        let mut multi = Multi::new();

        let mut easys :Vec<(Easy2Handle<Collector>, i32)> = Vec::new();
        multi.pipelining(true, true).unwrap();
        for (url, easykey) in self.async_state.url_key.iter() {
            easys.push((init(&multi, url).unwrap(), *easykey));
        }
        while multi.perform().unwrap() > 0 {
            multi.wait(&mut [], Duration::from_secs(10)).unwrap();
        }

        for easyhandler in easys.drain(..) {
            let mut handler_after = multi.remove2(easyhandler.0).unwrap();
            let easykey = easyhandler.1;
            let resp_code = handler_after.response_code().unwrap();

            let (pkg, pkg_num, version) = self.async_state.key_pkg.get(&easykey).unwrap();
            let data = handler_after.get_ref().0.clone();
            if resp_code == 200 {
                let data = handler_after.get_ref().0.clone();
                let local_md5 = self.md5sums.get(pkg_num).unwrap();
                if data.trim() == local_md5 {
                    println!("verifying {}, matches: {:?}", pkg, true);
                }
                else {
                    println!("verifying {}, matches: {:?}", pkg, true);
                }
            }
            else {
                println!("got error {} on request for package {} version {}", resp_code, pkg, version);
            }
            self.async_state.key_pkg.remove(&easykey);
        }

        // println!("verifying {}, matches: {:?}", pkg, same_md5sum);
        // println!("got error {} on request for package {} version {}", c, ..., ...
    }
}
fn init(multi: &Multi, url:&str) -> Result<Easy2Handle<Collector>, curl::Error> {
    let mut easy = Easy2::new(Collector(Box::new(String::new())));
    easy.url(url).unwrap();
    easy.verbose(false)?;
    Ok(multi.add2(easy).unwrap())
}
