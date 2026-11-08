use bartap::model::Settings;

#[test]
fn legacy_material_settings_are_retired() {
    let path = std::env::temp_dir().join(format!("bartap-black-skin-{}", std::process::id()));
    std::fs::create_dir_all(&path).unwrap();
    std::env::set_var("BARTAP_DATA_DIR", &path);
    for skin in ["glass", "clear", "black"] {
        std::fs::write(path.join("settings.txt"), format!("material={skin}\nminimal=0\n")).unwrap();
        let settings = Settings::load();
        settings.save();
        let saved = std::fs::read_to_string(path.join("settings.txt")).unwrap();
        assert!(!saved.contains("material="));
        assert!(saved.contains("minimal=0"));
    }
    std::fs::remove_file(path.join("settings.txt")).unwrap();
    std::fs::remove_dir(path).unwrap();
}
