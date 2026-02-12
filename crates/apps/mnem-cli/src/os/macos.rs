use anyhow::{anyhow, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn install_url_handler() -> Result<()> {
    let exe_path = std::env::current_exe()?;
    let exe_str = exe_path.to_string_lossy();

    let applescript = format!(
        r#"
on open location this_URL
    try
        set AppleScript's text item delimiters to "open/"
        set theItems to text items of this_URL
        if (count of theItems) < 2 then
            display notification "Invalid mnem URL" with title "Mnemosyne"
            return
        end if
        set theHash to item 2 of theItems
        
        -- Run mnem open
        do shell script "\"{}\" open " & theHash
    on error errMsg
        display dialog "Mnem Error: " & errMsg buttons {{"OK"}} default button "OK"
    end try
end open location
"#,
        exe_str
    );

    let script_path = PathBuf::from("/tmp/mnem_handler.applescript");
    fs::write(&script_path, applescript)?;

    let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
    let app_path = home.join("Applications/MnemHandler.app");

    Command::new("osacompile")
        .arg("-o")
        .arg(&app_path)
        .arg(&script_path)
        .status()?;

    let plist_path = app_path.join("Contents/Info.plist");
    let mut plist_content = fs::read_to_string(&plist_path)?;
    let url_scheme_dict = r#"
	<key>CFBundleURLTypes</key>
	<array>
		<dict>
			<key>CFBundleURLName</key>
			<string>Mnem Handler</string>
			<key>CFBundleURLSchemes</key>
			<array>
				<string>mnem</string>
			</array>
		</dict>
	</array>"#;

    if let Some(pos) = plist_content.rfind("</dict>") {
        plist_content.insert_str(pos, url_scheme_dict);
        fs::write(&plist_path, plist_content)?;
    }

    // Register and clean quarantine
    let lsregister_path = "/System/Library/Frameworks/CoreServices.framework/Versions/A/Frameworks/LaunchServices.framework/Versions/A/Support/lsregister";
    Command::new(lsregister_path)
        .arg("-f")
        .arg(&app_path)
        .status()
        .ok();
    Command::new("xattr")
        .arg("-d")
        .arg("com.apple.quarantine")
        .arg(&app_path)
        .status()
        .ok();

    Ok(())
}
