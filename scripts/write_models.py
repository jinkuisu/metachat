import json
with open("F:/metacaht/scripts/models_data.json") as f:
    chunks = json.load(f)
with open("F:/metacaht/desktop/src-tauri/src/storage/models.rs","w") as f:
    for c in chunks:
        f.write(c + chr(10))
print("Done: " + str(len(chunks)) + " chunks")
