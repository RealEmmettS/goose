fn main() {
    println!("cargo:rerun-if-changed=Assets/UI/honk300-app.rc");
    println!("cargo:rerun-if-changed=Assets/UI/honk300-app.ico");
    embed_resource::compile_for(
        "Assets/UI/honk300-app.rc",
        ["honk300-app"],
        embed_resource::ParamsIncludeDirs(["Assets/UI"]),
    )
    .manifest_required()
    .unwrap();
}
