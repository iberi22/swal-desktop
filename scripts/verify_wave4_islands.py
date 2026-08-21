import glob
import os
import re

files = sorted(glob.glob('.hermes/ola4/body-*.md'))
if len(files) != 10:
    raise ValueError(f"Expected 10 bodies, found {len(files)}")

required_sections = [
    "Current State",
    "Desired State",
    "Web Research Required",
    "Acceptance Criteria",
    "Files to Modify",
    "DO NOT touch",
    "Anti-Hallucination Guard",
    "Merge Order"
]

file_islands = {}

for fpath in files:
    content = open(fpath).read()
    for sec in required_sections:
        if sec not in content:
            raise ValueError(f"Missing section '{sec}' in {fpath}")
    
    # Extract files to modify from table
    matches = re.findall(r'\|\s*`([^`]+)`\s*\|', content)
    file_islands[fpath] = matches
    print(f"✅ {os.path.basename(fpath)}: verified sections, target files: {matches}")

# Check for disjointness
intersections = []
fpaths = list(file_islands.keys())
for i in range(len(fpaths)):
    for j in range(i + 1, len(fpaths)):
        f1, f2 = fpaths[i], fpaths[j]
        common = set(file_islands[f1]) & set(file_islands[f2])
        if common:
            intersections.append((f1, f2, common))

if intersections:
    print("❌ Intersection errors:", intersections)
    exit(1)
else:
    print("\n🎉 100% DISJOINT FILE ISLANDS VERIFIED FOR WAVE 4! (0 Intersections across 10 tasks)")
