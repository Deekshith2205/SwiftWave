import os
from pathlib import Path

replacements = {
    "nexus_share_app": "swiftwave_app",
    "nexusShareApp": "swiftwaveApp",
    "com.nexusshare": "com.swiftwave",
    "Nexus_Share": "SwiftWave"
}

files_to_process = [
    "apps/flutter_app/.idea/modules.xml",
    "apps/flutter_app/android/app/src/main/AndroidManifest.xml",
    "apps/flutter_app/linux/CMakeLists.txt",
    "apps/flutter_app/linux/runner/my_application.cc",
    "apps/flutter_app/macos/Flutter/ephemeral/Flutter-Generated.xcconfig",
    "apps/flutter_app/macos/Runner/Configs/AppInfo.xcconfig",
    "apps/flutter_app/macos/Runner.xcodeproj/project.pbxproj",
    "apps/flutter_app/macos/Runner.xcodeproj/xcshareddata/xcschemes/Runner.xcscheme",
    "apps/flutter_app/windows/CMakeLists.txt",
    "apps/flutter_app/windows/flutter/ephemeral/generated_config.cmake",
    "apps/flutter_app/windows/runner/main.cpp",
    "apps/flutter_app/windows/runner/Runner.rc"
]

base_dir = Path("E:/SwiftWave")

for file_path in files_to_process:
    full_path = base_dir / file_path
    if not full_path.exists():
        print(f"File not found: {full_path}")
        continue
    
    content = full_path.read_text(encoding="utf-8")
    original_content = content
    
    # Apply replacements in order (more specific first)
    for old, new in replacements.items():
        content = content.replace(old, new)
        
    if content != original_content:
        full_path.write_text(content, encoding="utf-8")
        print(f"Updated {file_path}")
    else:
        print(f"No changes in {file_path}")
