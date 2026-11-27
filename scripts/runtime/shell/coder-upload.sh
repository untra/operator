#!/bin/sh
bin_dir="__OP_BIN__"
mkdir -p "$bin_dir" &&
    cat > "$bin_dir/opr8r.tmp" &&
    chmod 700 "$bin_dir/opr8r.tmp" &&
    mv -f "$bin_dir/opr8r.tmp" "$bin_dir/opr8r"
