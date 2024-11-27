librime_dir="librime/build/bin/"
code="`pwd`/$1"
output="`pwd`/$2"

(
    cd $librime_dir
    ./rime_api_console < $code 2> /dev/null | grep commit | sed 's/commit\: //' > $output
)
