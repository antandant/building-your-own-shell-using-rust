use std::{
    env,
    io::{stdin, stdout, Write},
    path::Path,
    process::{Child, Command, Stdio},
};
fn main() {
    loop {
        // 入力プロンプトとして`>`を使う
        // read_lineより前に`>`を表示させるために，明示的にflushする必要がある
        print!("> ");
        stdout().flush().unwrap();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        // パイプラインの実装
        let mut commands = input.trim().split(" | ").peekable(); // パイプで分ける
        let mut previous_command = None;

        while let Some(command) = commands.next() {
            // コマンドを空白区切りにし，１つ目をコマンド本体，それ以降を引数とみなす
            let mut parts = command.trim().split_whitespace();
            let command = parts.next().unwrap();
            let args = parts;

            match command {
                "cd" => {
                    // cdコマンドを実装する
                    // 新しいディレクトリが提供されていない場合はデフォルトで '/' を新しいディレクトリとして使う
                    let new_dir = args.peekable().peek().map_or("/", |x| *x);
                    let root = Path::new(new_dir);
                    if let Err(e) = env::set_current_dir(&root) {
                        eprintln!("{}", e);
                    }

                    previous_command = None;
                }
                "exit" => return, // exitコマンドを実装
                command => {
                    let stdin = previous_command.map_or(Stdio::inherit(), |output: Child| {
                        Stdio::from(output.stdout.unwrap())
                    });
                    let stdout = if commands.peek().is_some() {
                        // このコマンドの次にまだコマンドがある
                        // パイプラインの次のコマンドに出力を渡す
                        Stdio::piped()
                    } else {
                        // このコマンドの次にもうコマンドがない
                        Stdio::inherit()
                    };

                    let mut output = Command::new(command)
                        .args(args)
                        .stdin(stdin)
                        .stdout(stdout)
                        .spawn(); // 入力されたコマンドを実行する
                                  // 存在しないコマンドがあればエラーを表示してシェルを続行する
                    match output {
                        Ok(output) => {
                            previous_command = Some(output); // この子プロセスが終了するまでユーザーに入力を促さないようにする
                        }
                        Err(e) => {
                            previous_command = None;
                            eprintln!("{}", e);
                        }
                    };
                }
            }
        }

        if let Some(mut final_command) = previous_command {
            final_command.wait();
        }
    }
}
