import sys

def main():
    file_path = 'models/src/lib.rs'
    with open(file_path, 'r') as f:
        content = f.read()

    old_text = """    /// Delegate tool execution to the Tool Provider extension.
    ExecuteTool {
        call_id: String,
        name: String,
        arguments: String,
    },"""
    
    new_text = """    /// Delegate tool execution to the Tool Provider extension.
    ExecuteTool {
        call_id: String,
        name: String,
        arguments: String,
    },
    /// Call a specific method on a Wasm extension.
    CallExtension {
        extension_id: String,
        method: String,
        arguments: String,
    },"""

    if old_text in content:
        new_content = content.replace(old_text, new_text)
        with open(file_path, 'w') as f:
            f.write(new_content)
        print("Successfully updated models/src/lib.rs")
    else:
        print("Old text not found in models/src/lib.rs")
        print("--- Content start ---")
        print(content[:500])
        print("--- Content end ---")
        sys.exit(1)

if __name__ == "__main__":
    main()
