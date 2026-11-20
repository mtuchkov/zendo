use super::*;

/// The variables of `text`, with `HOME=/h` and `PATH=/bin` in the environment of zendo, and the
/// numbers of the skipped lines.
fn read(text: &str) -> (Vec<(String, String)>, Vec<usize>) {
    let env = |k: &str| match k {
        "HOME" => Some("/h".to_string()),
        "PATH" => Some("/bin".to_string()),
        _ => None,
    };
    let (vars, skipped) = read_envrc(text, &env, Path::new("/h"));
    (vars.into_iter().collect(), skipped)
}

fn pairs(want: &[(&str, &str)]) -> Vec<(String, String)> {
    want.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

// ENV-4: a line that the shell would run, or that is not one whole assignment, is no variable.
#[test]
fn lines_that_run_code_are_skipped() {
    let (vars, skipped) = read("A=$(id)\nB=`id`\nC=1; id\nD=a b\n1E=2\nexport\nF=\"a`id`\"\nexportG=1\nexport\tH=2\nI='a'\"b\"\nJ=a\\ b\n");
    assert_eq!(vars, pairs(&[("H", "2"), ("exportG", "1")]));
    assert_eq!(skipped, [1, 2, 3, 4, 5, 6, 7, 10, 11]);
}

// ENV-8: zendo puts the value of `$NAME` and `${NAME}` and the home folder for `~` itself: a
// variable of the file that stands above, else of the environment of zendo, else empty.
#[test]
fn values_expand_variables_and_the_home_folder() {
    let text = "export PATH=\"$HOME/bin:$PATH\"\nB=${HOME}x\nC=$PATH:~/c\nT=~/t\nU=~\nN=\"<$NOT_SET>\"\nQ='$HOME ~'\nE=\"a\\\"b\\$HOME ~\"\nW=x~y\n";
    let (vars, skipped) = read(text);
    let want = [("B", "/hx"), ("C", "/h/bin:/bin:/h/c"), ("E", "a\"b$HOME ~"), ("N", "<>"), ("PATH", "/h/bin:/bin"), ("Q", "$HOME ~"), ("T", "/h/t"), ("U", "/h"), ("W", "x~y")];
    assert_eq!(vars, pairs(&want));
    assert_eq!(skipped, [0usize; 0]);
}

// ENV-8: an expansion that only a shell can do makes the line no variable.
#[test]
fn an_expansion_that_needs_a_shell_is_skipped() {
    let (vars, skipped) = read("A=${HOME:-x}\nB=$1\nC=\"$(id)\"\nD=$?\nE=${HOME\nF=ok$\n");
    assert_eq!(vars, pairs(&[("F", "ok$")]));
    assert_eq!(skipped, [1, 2, 3, 4, 5]);
}

// ENV-4: a comment can stand after the value.
#[test]
fn a_comment_can_stand_after_the_value() {
    let (vars, skipped) = read("A=1 # one\nB=\"two\"\t# two\nC='3'#x\nD=4#5\n");
    assert_eq!(vars, pairs(&[("A", "1"), ("B", "two"), ("D", "4#5")]));
    assert_eq!(skipped, [3]);
}

// ENV-9: an assignment in a block of the shell depends on what the shell does, so zendo takes
// none of the block: a condition, a loop, a function, a here-document, a text of several lines,
// and a line that goes on in the next one.
#[test]
fn lines_in_a_block_are_skipped() {
    let text = "\
if [ -n \"$CI\" ]; then
  export WHO=ci
else
  export WHO=me
fi
A=1
if true; then export B=1; fi
case $x in
  a) C=1 ;;
esac
f() {
  D=1
}
for i in 1 2; do
E=1
done
cat <<END
F=1
END
G=\"first
H=1
last\"
I=1 \\
J=1
Z=9
";
    let (vars, skipped) = read(text);
    assert_eq!(vars, pairs(&[("A", "1"), ("Z", "9")]));
    assert_eq!(skipped, [1, 2, 3, 4, 5, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24]);
}

// ENV-7: the message names the lines in short: a run of lines as its first and its last.
#[test]
fn the_message_names_the_runs_of_lines() {
    assert_eq!(skipped_problem(Path::new("/r/.envrc"), &[]), None);
    let one = skipped_problem(Path::new("/r/.envrc"), &[5]).unwrap();
    assert!(one.contains("/r/.envrc") && one.contains("skipped line 5:"), "{one}");
    let more = skipped_problem(Path::new("/r/.envrc"), &[1, 2, 3, 5, 8, 9]).unwrap();
    assert!(more.contains("skipped lines 1-3, 5, 8-9:"), "{more}");
}

/// The names of the variables that zendo takes from `text`.
fn names(text: &str) -> Vec<String> {
    read(text).0.into_iter().map(|(k, _)| k).collect()
}

// ENV-9: more forms of a block: a function without a space before the brace and with a comment
// after it, a line that ends with `&&`, `||` or `|` and goes on in the next one, and a list in
// parentheses over several lines, which does not close a condition.
#[test]
fn more_forms_of_a_block_are_skipped() {
    assert_eq!(names("use_x(){\nexport A=1\n}\nB=1\n"), ["B"]);
    assert_eq!(names("f() { # why\nexport A=1\n}\nB=1\n"), ["B"]);
    assert_eq!(names("[ -f .env.local ] &&\nexport A=1\nB=1\n"), ["B"]);
    assert_eq!(names("false ||\nexport A=1\nB=1\n"), ["B"]);
    assert_eq!(names("cat x |\nA=1\nB=1\n"), ["B"]);
    assert_eq!(names("if true; then\nFLAGS=(\n-a\n)\nexport A=1\nfi\nB=1\n"), ["B"]);
    assert_eq!(names("R=$(\nIFS=,\necho x\n)\nB=1\n"), ["B"]);
    assert_eq!(names("(\nA=1\n)\nB=1\n"), ["B"]);
}

// ENV-8: zendo does not know the value of a variable whose line it skipped, so a line that uses
// such a variable is no variable line: its value would be wrong. A value that a line outside a
// block set stays the value. `~name` needs the shell.
#[test]
fn a_variable_of_a_skipped_line_has_no_value() {
    let (vars, skipped) = read("TOOLS=$(realpath tools)\nexport PATH=\"$TOOLS:$PATH\"\nB=1\n");
    assert_eq!((vars, skipped), (pairs(&[("B", "1")]), vec![1, 2]));
    let (vars, skipped) = read("A=1\nA=$(id)\nC=$A\n");
    assert_eq!((vars, skipped), (pairs(&[]), vec![2, 3]), "a new value that zendo cannot read takes the old one away");
    assert_eq!(names("if x; then\nROOT=/a\nfi\nCFG=$ROOT/c\nB=1\n"), ["B"]);
    assert_eq!(read("ROOT=/top\nif x; then\nROOT=/a\nfi\nCFG=$ROOT/c\n").0, pairs(&[("CFG", "/top/c"), ("ROOT", "/top")]));
    assert_eq!(read("U=~other/x\nB=1\n"), (pairs(&[("B", "1")]), vec![1]));
}

// ENV-9: a text in quotes and a list in parentheses end where their quote or their parenthesis
// closes, also in the middle of a line, and the variable lines after them count. The words in a
// text in quotes open and close no block.
#[test]
fn a_block_ends_where_it_closes() {
    assert_eq!(names("export CREDS='{\n  \"type\": \"service_account\"\n}'\nexport NEXT=1\n"), ["NEXT"]);
    assert_eq!(names("export CREDS=\"{\n  x\n}\"\nexport NEXT=1\n"), ["NEXT"]);
    assert_eq!(names("export URL=\"$(\n  op read x\n)\"\nexport NEXT=1\n"), ["NEXT"]);
    assert_eq!(names("PKGS=(\n  foo\n  bar)\nexport NEXT=1\n"), ["NEXT"]);
    assert_eq!(names("V=$(\n  cat VERSION)\nNEXT=1\n"), ["NEXT"]);
    assert_eq!(names("(cd sub\n A=1\n)\nNEXT=1\n"), ["NEXT"]);
    assert_eq!(names("if (\ntrue\n); then\nexport A=1\nfi\nNEXT=1\n"), ["NEXT"]);
    assert_eq!(names("for f in $(\nls\n); do\nexport A=$f\ndone\nNEXT=1\n"), ["NEXT"]);
    assert_eq!(names("A=`\nB=1\n`\nNEXT=1\n"), ["NEXT"]);
    assert_eq!(names("if x; then\necho \"a ; fi ; b\"\nA=1\nfi\nNEXT=1\n"), ["NEXT"]);
    assert_eq!(names("echo \"a\nb\" &&\nA=1\nNEXT=1\n"), ["NEXT"]);
    assert_eq!(names("cat <<'END'\nA=1\nEND\nNEXT=1\n"), ["NEXT"]);
}

// ENV-8: a skipped line can set a variable in more ways than `NAME=value`, and zendo knows the
// value of none of them. A value for one command, `NAME=value command`, does not change the
// variable.
#[test]
fn a_skipped_line_sets_variables_in_more_ways() {
    let unknown = |text: &str| read(text).0.into_iter().map(|(k, _)| k).collect::<Vec<_>>();
    assert_eq!(unknown("declare -x ROOT=$(pwd)\nCFG=\"$ROOT/.cfg\"\nB=1\n"), ["B"]);
    assert_eq!(unknown(": \"${PORT:=8080}\"\nURL=\"http://localhost:$PORT\"\nB=1\n"), ["B"]);
    assert_eq!(unknown("read -r TOKEN < f\nAUTH=\"Bearer $TOKEN\"\nB=1\n"), ["B"]);
    assert_eq!(unknown("export A=$(pwd) B=1\nC=\"$B/x\"\nD=1\n"), ["D"]);
    assert_eq!(unknown("for f in a b; do :; done\nC=$f\nD=1\n"), ["D"]);
    assert_eq!(unknown("P=1\nP+=2\nC=$P\nD=1\n"), ["D"]);
    let (vars, skipped) = read("export NODE_ENV=development\nNODE_ENV=test npm run build\nMODE=$NODE_ENV\n");
    assert_eq!((vars, skipped), (pairs(&[("MODE", "development"), ("NODE_ENV", "development")]), vec![2]));
}
