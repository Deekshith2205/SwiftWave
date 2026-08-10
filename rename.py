import os
from pathlib import Path

replacements = {
    "NEXUS Share": "SwiftWave",
    "NEXUS SHARE": "SWIFTWAVE",
    "Nexus Share": "SwiftWave",
    "nexus_share": "swiftwave",
    "nexus-share": "swiftwave",
    "nexus_core": "swiftwave_core",
    "nexus_ffi": "swiftwave_ffi",
    "nexus_discovery": "swiftwave_discovery",
    "nexus_storage": "swiftwave_storage",
    "NexusDiscovery": "SwiftWaveDiscovery",
    "NexusJni": "SwiftWaveJni",
    "NEXUS_LOG": "SWIFTWAVE_LOG",
    "NEXUS_TRANSPORT": "SWIFTWAVE_TRANSPORT",
    "NEXUS_DISCOVERY": "SWIFTWAVE_DISCOVERY",
    "com.nexusshare": "com.swiftwave",
    "nexusshare": "swiftwave",
    "NEXUS": "SwiftWave", # Careful with this one, do it last
}

files_to_process = [
    "native/android/SwiftWaveDiscovery.kt",
    "native/android/README.md",
    "native/linux/swiftwave_discovery_linux.rs",
    "native/linux/README.md",
    "native/macos/swiftwave_discovery_macos.rs",
    "native/macos/README.md",
    "native/windows/swiftwave_discovery_win.rs",
    "native/windows/README.md",
    "docs/ARCHITECTURE.md",
    "docs/BUILD.md",
    "docs/PLATFORM_MATRIX.md",
    "docs/ROADMAP.md",
    "docs/SECURITY.md",
    "tests/README.md",
    "scripts/build_all.ps1",
    "scripts/setup.ps1",
    "scripts/setup.sh",
    "README.md"
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
