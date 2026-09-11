#!/usr/bin/env bash
# EWW file manager toggle — PRIMARY shell (Rust swal-files is dev fallback)
if eww active-windows 2>/dev/null | grep -q "swal_files"; then
  eww close swal_files 2>/dev/null; eww close swal_files_maximized 2>/dev/null; eww close swal_editor 2>/dev/null &
else
  eww open swal_files 2>/dev/null || true
fi
