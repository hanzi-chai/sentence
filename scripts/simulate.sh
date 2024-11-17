librime_dir="librime/build/bin/"
code=`realpath $1`
output="`pwd`/$2"

cp $code $librime_dir

(
    cd $librime_dir
    ./rime_deployer --build
    ./rime_api_console < $code | grep commit | sed 's/commit\: //' > $output
)
