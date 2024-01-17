
import std/os
import std/strutils
import std/macros

const
  currentDir = currentSourcePath().parentDir()
  libDir* = currentDir/"target"/"release"
  libPath* = libDir/"libcodex_storage_proofs.a"

static:
  let cmd = "cargo build --release"
  warning "\nBuilding codex-storage-proofs: " & cmd
  let (output, exitCode) = gorgeEx cmd
  for ln in output.splitLines():
    warning("cargo> " & ln)
  if exitCode != 0:
    raise (ref Defect)(msg: "Failed to build codex-storage-proofs")


{.passl: "-lcodex_storage_proofs" & " -L" & libDir.}

include codex_proofs_ffi

proc len*(buff: Buffer): int =
  buff.len.int

template unsafeBufferPath*(path: var string): Buffer =
  assert path.len() > 0
  Buffer(data: cast[ptr uint8](path.cstring),
         len: path.len().uint)

template unsafeBufferFromFile*(path: string): Buffer =
  assert path.len() > 0
  let entireFile = readFile(path)

  Buffer(data: cast[ptr uint8](entireFile.cstring),
         len: entireFile.len().uint)
