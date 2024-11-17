table=$1
librime_dir="librime/build/bin/"

cp assets/essay.txt $librime_dir
cp test.schema.yaml $librime_dir
cat <<EOF > "${librime_dir}default.custom.yaml"
patch:
  schema_list:
    - schema: test
EOF
cat <<EOF > "${librime_dir}test.dict.yaml"
---
name: test
version: 1.0.0
sort: by_weight
use_preset_vocabulary: true
...

$(cat "$table")
EOF
