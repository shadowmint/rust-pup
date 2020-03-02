# pup-worker-pathtools

Apply standardized actions to specific paths.

Currently the two operations which are supported are:

    - (Create) Create a new folder, if it doesn't exist.
    - (Require) Create a new folder, leaving it alone if it exists.
    - (Archive) If a folder exists, move it to [old-name]-[GUID]
    
The manifest should be a series of operations to perform, as follows:

```
steps:
  - path: "old_folder"
    action: Archive

  - path: "old_folder"
    action: Create

  - path: "new_folder"
    action: Require
```