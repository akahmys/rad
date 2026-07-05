(module
  (import "env" "rad_host_rpc" (func $host_rpc (param i32 i32) (result i64)))
  (memory (export "memory") 1)
  (global $alloc_ptr (mut i32) (i32.const 1024))
  (global $test_case (mut i32) (i32.const 0))

  (func (export "alloc") (param $size i32) (result i32)
    (local $ptr i32)
    (local.set $ptr (global.get $alloc_ptr))
    (global.set $alloc_ptr (i32.add (local.get $ptr) (local.get $size)))
    (local.get $ptr)
  )

  (func (export "dealloc") (param $ptr i32) (param $size i32)
    ;; no-op
  )

  (func (export "set_test_case") (param $val i32)
    (global.set $test_case (local.get $val))
  )

  (func (export "rad_on_event") (param $event_ptr i32) (param $event_len i32) (result i64)
    (local $case i32)
    (local.set $case (global.get $test_case))

    ;; Case 0: FileRead (test.txt) (len: 58)
    (if (i32.eq (local.get $case) (i32.const 0))
      (then
        (call $host_rpc (i32.const 0) (i32.const 58))
        (drop)
      )
    )

    ;; Case 1: FileWrite (test_write.txt) (len: 95)
    (if (i32.eq (local.get $case) (i32.const 1))
      (then
        (call $host_rpc (i32.const 100) (i32.const 95))
        (drop)
      )
    )

    ;; Case 2: SpawnBashProcess (echo hello) (len: 72)
    (if (i32.eq (local.get $case) (i32.const 2))
      (then
        (call $host_rpc (i32.const 200) (i32.const 72))
        (drop)
      )
    )

    ;; Case 3: SpawnBashProcess Blocked (curl ...) (len: 85)
    (if (i32.eq (local.get $case) (i32.const 3))
      (then
        (call $host_rpc (i32.const 300) (i32.const 85))
        (drop)
      )
    )

    ;; Case 4: Create DAG Node (len: 77)
    (if (i32.eq (local.get $case) (i32.const 4))
      (then
        (call $host_rpc (i32.const 400) (i32.const 77))
        (drop)
      )
    )

    ;; Case 5: GetDag (len: 49)
    (if (i32.eq (local.get $case) (i32.const 5))
      (then
        (call $host_rpc (i32.const 500) (i32.const 49))
        (drop)
      )
    )

    (i64.const 0)
  )

  ;; JSON RPC Data Sections
  ;; Case 0: FileRead test.txt (len: 58)
  (data (i32.const 0) "{\"id\":\"1\",\"method\":\"FileRead\",\"params\":{\"path\":\"test.txt\"}}")

  ;; Case 1: FileWrite test_write.txt (len: 94)
  (data (i32.const 100) "{\"id\":\"2\",\"method\":\"FileWrite\",\"params\":{\"path\":\"test_write.txt\",\"data\":[104,101,108,108,111]}}")

  ;; Case 2: SpawnBashProcess echo hello (len: 72)
  (data (i32.const 200) "{\"id\":\"3\",\"method\":\"SpawnBashProcess\",\"params\":{\"command\":\"echo hello\"}}")

  ;; Case 3: SpawnBashProcess Blocked (curl) (len: 85)
  (data (i32.const 300) "{\"id\":\"4\",\"method\":\"SpawnBashProcess\",\"params\":{\"command\":\"curl http://example.com\"}}")

  ;; Case 4: CreateNode (len: 76)
  (data (i32.const 400) "{\"id\":\"5\",\"method\":\"CreateNode\",\"params\":{\"parent_id\":\"\",\"node_type\":\"task\"}}")

  ;; Case 5: GetDag (len: 49)
  (data (i32.const 500) "{\"id\":\"6\",\"method\":\"GetDag\",\"params\":null}")
)
