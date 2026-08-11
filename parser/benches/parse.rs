use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use std::fs::read;
use std::hint::black_box;
use std::path::Path;
use tf_demo_parser::demo::data::DemoTick;
use tf_demo_parser::demo::message::Message;
use tf_demo_parser::demo::parser::analyser::Analyser;
use tf_demo_parser::demo::parser::gamestateanalyser::GameStateAnalyser;
use tf_demo_parser::demo::parser::MessageHandler;
use tf_demo_parser::{Demo, DemoParser, MessageType, ParserState};

struct AllMessages;

impl MessageHandler for AllMessages {
    type Output = bool;

    fn does_handle(_message_type: MessageType) -> bool {
        true
    }

    fn handle_message(&mut self, message: &Message, _tick: DemoTick, _parser_state: &ParserState) {
        black_box(message);
    }

    fn into_output(self, _state: &ParserState) -> Self::Output {
        black_box(true)
    }
}

fn read_file<P: AsRef<Path>>(path: P) -> Demo<'static> {
    let data = read(path).unwrap();
    Demo::owned(data)
}

fn get_parser<P: AsRef<Path>, A: MessageHandler>(path: P, analyser: A) -> DemoParser<'static, A> {
    let demo = read_file(path);
    let stream = demo.get_stream();
    DemoParser::new_with_analyser(stream.clone(), analyser)
}

#[library_benchmark(setup = get_parser)]
#[bench::basic_small("test_data/small.dem", Analyser::new())]
#[bench::basic_gully("test_data/gully.dem", Analyser::new())]
#[bench::game_state_small("test_data/small.dem", GameStateAnalyser::new())]
#[bench::game_state_gully("test_data/gully.dem", GameStateAnalyser::new())]
#[bench::all_small("test_data/small.dem", AllMessages)]
#[bench::all_gully("test_data/gully.dem", AllMessages)]
fn parse<A: MessageHandler>(parser: DemoParser<A>) {
    black_box(parser.parse().unwrap());
}

library_benchmark_group!(
    name = bench_parse;
    benchmarks = parse
);

main!(library_benchmark_groups = bench_parse);
