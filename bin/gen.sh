#!/bin/bash


if [ ! -d "schemas" ]; then
  echo "No schemas directory found"
  exit 1
fi

if [ ! -x $(which flatc) ]; then
  echo "Flatbuffers compiler is required"
  exit 1
fi

flatc --ts --gen-object-api --gen-mutable --reflect-names -o schemas-ts schemas/*.fbs
sed -i.bak '
  /mutate_writable(value:boolean):boolean {/,/}/ {
    s/this\.bb!\.writeInt8(this\.bb_pos + 0, value);/this.bb!.writeInt8(this.bb_pos + 0, value ? 1 : 0);/g
  }
' "schemas-ts/account.ts" && rm "schemas-ts/account.ts.bak"

pnpm --filter bonsol-schemas run build 