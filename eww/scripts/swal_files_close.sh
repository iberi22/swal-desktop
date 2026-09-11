#!/usr/bin/env bash
timeout 5 eww close swal_files 2>/dev/null || true
timeout 5 eww close swal_files_maximized 2>/dev/null || true
rm -f /tmp/swal_files_visible.flag /tmp/swal-files.pid
