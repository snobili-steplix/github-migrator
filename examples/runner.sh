while read in; do
    ../target/release/github-migrator $in;
done < $1