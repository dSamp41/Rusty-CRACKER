use concurrent_graph::ConcurrentDiGraph;
use getopts::Options;
mod concurrent_graph;

//use petgraph::graphmap::{DiGraphMap, UnGraphMap};
use std::{collections::HashSet, env};

mod concurrentgraph_utils_rayon;
use concurrentgraph_utils_rayon::{min_selection_ep, par_seed_propagation, prune_os, seed_propagation};

//mod graphmap_utils_rayon_v2;


mod io_util;
use io_util::read_from_file;
use rayon::ThreadPoolBuilder;

// ~20 ms / 50k edges

macro_rules! debug_println {
    ($($arg:tt)*) => (if ::std::cfg!(debug_assertions) { ::std::println!($($arg)*); })
}

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    type V = u32;

    //get cli args
    let args: Vec<String> = std::env::args().collect();

    //get opts
    let mut opts = Options::new();
    opts.optopt("f", "file", "provide the file containg the graph output file name", "FILEPATH");
    opts.optopt("n", "num_thread", "provide the number of threads to use", "0");
    opts.optflag("h", "help", "print help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(matches) => matches,
        Err(fail) => {
            panic!("{}", fail.to_string())
        }
    };

    //handle -h/--help
    if matches.opt_present("h") {
        let brief = format!("Usage: {} FILE [options]", args[0]);
        print!("{}", opts.usage(&brief));

        return;
    }

    //handle -n/--num_threads
    let num_threads = match matches.opt_str("n") {
        None => 0,  //let rayon decide
        Some(v) => v.parse().unwrap(),
    };

    //setup parallelism
    ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .unwrap();


    //handle -f/--filename
    let filename = matches.opt_str("f");
    if filename.is_none() {
        println!("Please provide a filename");
        return;
    }

    let edges_result = read_from_file::<V>(filename.unwrap().as_str());
    if edges_result.is_err() {
        println!("Error reading edges from file: {:?}", edges_result.err());
        return;
    }

    let edges: Vec<(V, V)> = edges_result.unwrap_or_default();
    let graph = ConcurrentDiGraph::<V>::new_directed();


    //TODO: parallelize graph creation
    for edge in edges {
        graph.add_edge(edge.0, edge.1);
        graph.add_edge(edge.1, edge.0);
    }



    let tree = ConcurrentDiGraph::<V>::new_directed();

    let mut gt = graph.clone();
    let mut t = tree.clone();

    let mut num_it = 1;


    let now = std::time::Instant::now();

    loop {
        //min selection
        let h: ConcurrentDiGraph<V> = min_selection_ep(&gt);
        //debug_println!("h_{:?} #edges: {:?}", num_it, gt.edge_count());
        debug_println!("@ min_selection_{num_it}: {:?}", now.elapsed());

        //pruning
        let (temp_g, tree) = prune_os(h, t);
        debug_println!("@ pruning_{num_it}: {:?}", now.elapsed());

        gt = temp_g;
        //println!("g{num_it}: {:?}", gt);
        t = tree;

        if gt.node_count() == 0 {
            break;
        }

        num_it += 1;
        //debug_println!("g_{:?} #edges: {:?}", num_it, gt.edge_count());
    }


    println!("{:?}", now.elapsed().as_millis());

    let seeds = seed_propagation(&t);
    println!("duration: {:?}", now.elapsed());

    debug_println!("t: {num_it}");
    //assert_eq!(seeds.len(), graph.node_count()); //all node have a seed => no nodes are lost
    //println!("seeds: {seeds:?}");

    //let num_conn_comp: HashSet<_> = seeds.values().collect();
    //debug_println!("#CC: {:?}", num_conn_comp.len());

    debug_println!("end: {:?}", now.elapsed());
    //println!("seeds: {:?}", num_conn_comp);


}