fn main() {
    println!("cargo:rustc-env=LISTENPORT=1234");
    println!("cargo:rustc-env=LISTENINTERFACE=0.0.0.0");
    println!("cargo:rustc-env=REMOTEPROTOCOL=https://");
    println!("cargo:rustc-env=REMOTEURL=donald.remote.server");
    println!("cargo:rustc-env=REMOTEPATH=/donald/whitehouse");
    println!("cargo:rustc-env=DEBUG=false");
}
