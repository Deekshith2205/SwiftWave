import os
from pathlib import Path

replacements = {
    "NexusAvatar": "SwiftWaveAvatar",
    "NexusPrimaryButton": "SwiftWavePrimaryButton",
    "NexusSecondaryButton": "SwiftWaveSecondaryButton",
    "NexusGhostButton": "SwiftWaveGhostButton",
    "NexusCard": "SwiftWaveCard",
    "NexusProgress": "SwiftWaveProgress",
    "NexusLinearProgress": "SwiftWaveLinearProgress",
    "NexusStatus": "SwiftWaveStatus",
    "NexusStatusBadge": "SwiftWaveStatusBadge",
    "NexusTheme": "SwiftWaveTheme",
    "NexusColors": "SwiftWaveColors",
    "NexusError": "SwiftWaveError",
    "nexus_avatar": "swiftwave_avatar",
    "nexus_buttons": "swiftwave_buttons",
    "nexus_cards": "swiftwave_cards",
    "nexus_progress": "swiftwave_progress",
    "nexus_status": "swiftwave_status",
    "nexus_core": "swiftwave_core",
    "NEXUS": "SwiftWave",
    "nexus share": "swiftwave",
    "Nexus Share": "SwiftWave",
    "nexus_share": "swiftwave"
}

base_dir = Path("E:/SwiftWave")

# Collect all files to process
import glob

files = []
for root, _, filenames in os.walk(base_dir):
    if ".git" in root or "build" in root or ".pub-cache" in root or ".dart_tool" in root or "target" in root:
        continue
    for filename in filenames:
        if filename.endswith(('.dart', '.rs', '.md', '.sh', '.ps1', '.xcconfig', '.cmake', '.cc', '.cpp')):
            files.append(os.path.join(root, filename))

for file_path in files:
    full_path = Path(file_path)
    try:
        content = full_path.read_text(encoding="utf-8")
    except:
        continue
    original_content = content
    
    # Apply replacements
    for old, new in replacements.items():
        content = content.replace(old, new)
        
    if content != original_content:
        full_path.write_text(content, encoding="utf-8")
        print(f"Updated {file_path}")
