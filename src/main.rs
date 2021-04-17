use {
  std::{
    fs,
    io::{
      self,
      BufWriter,
    },
    str::FromStr as _,
  },
  clap::{
    App, AppSettings, Arg, SubCommand, Shell
  },
};

mod gens;
mod utils;

fn mk_app() -> App<'static, 'static> {
  let mut app = App::new("hart")
    .version("1")
    .author("Nic Hartley <the@redfennec.dev>")
    .about("Randomly generated art")
    .setting(AppSettings::SubcommandRequiredElseHelp)
    .subcommand(SubCommand::with_name("completions")
      .about("Generate shell completions")
      .arg(Arg::with_name("shell")
        .possible_values(&Shell::variants())
        .default_value("zsh"))
      .arg(Arg::with_name("output")
        .short("o")
        .long("output")
        .help("The path to write the completion to")));

  for gen in &gens::Gen::all() {
    let subcmd = SubCommand::with_name(gen.command())
      .about(gen.about())
      .arg(Arg::with_name("seed")
        .long("seed")
        .help("Data to seed the random generator with")
        .takes_value(true))
      .arg(Arg::with_name("seed-file")
        .long("seed-file")
        .help("Path to a file containing the seed")
        .conflicts_with("seed")
        .takes_value(true))
      .arg(Arg::with_name("output")
        .short("o")
        .long("output")
        .help("Path to write the output to; extension automatically appended")
        .required(true)
        .takes_value(true));
    let added = gen.setup_cmd(subcmd);
    app = app.subcommand(added);
  }

  app
}

fn main() {
  let matches = mk_app().get_matches();

  let (subcmd, args) = matches.subcommand();
  let args = args.expect("How??");

  if subcmd == "completions" {
    // neither unwrap will panic: --shell has a default value and the only valid options are Shell::variants()
    let shell = Shell::from_str(args.value_of("shell").unwrap()).unwrap();
    match args.value_of("output") {
      Some(path) => {
        let file = fs::File::create(path).expect("Failed to open output");
        mk_app().gen_completions_to("hart", shell, &mut BufWriter::new(file));
      }
      None => {
        mk_app().gen_completions_to("hart", shell, &mut io::stdout())
      }
    }
  } else if let Some(gen) = gens::Gen::by_command(subcmd) {
    let seed = if let Some(data) = matches.value_of("seed") {
      data.as_bytes().to_vec()
    } else if let Some(path) = matches.value_of("seed-file") {
      fs::read(path).expect("Failed to open seed file")
    } else {
      println!("Enter some text as a seed:");
      let mut line = String::new();
      io::stdin().read_line(&mut line).expect("Failed to read input");
      line.as_bytes().to_vec()
    };

    let output_path = matches.value_of("output").unwrap();
    let mut output = BufWriter::new(fs::File::create(output_path).expect("Failed to open output"));

    gen.run(args, &seed, &mut output).expect("Failed to generate");
  } else {
    panic!("Invalid subcommand??");
  }
}
