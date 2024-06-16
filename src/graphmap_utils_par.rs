#![allow(dead_code)] 

use std::{collections::HashMap, fmt::Debug, sync::Mutex};
use dashmap::DashMap;
use petgraph::{ graphmap::{DiGraphMap, GraphMap, NodeTrait, UnGraphMap}, Direction::{Incoming, Outgoing}, EdgeType};
use rayon::prelude::*;

/// Get the neighborhood (plus itself) of every node
fn get_neighborhood<V, E, Ty>(g: &GraphMap<V, E, Ty>) -> DashMap<V, Vec<V>>
where
    V: NodeTrait + Send + Sync,
    E: Send + Sync,
    Ty: EdgeType + Send + Sync {
    let neighbors = DashMap::<V, Vec<V>>::new();
    
    let nodes: Vec<V> = g.nodes().collect();

    nodes.par_iter().for_each(|&node|{
        let mut node_neighbors: Vec<V> = g.neighbors(node).collect();
        node_neighbors.push(node);  //plus itself
        neighbors.insert(node, node_neighbors);
    });

    return neighbors;
}


fn get_neighborhood_base<V, E, Ty>(g: &GraphMap<V, E, Ty>) -> DashMap<V, Vec<V>>
where
    V: NodeTrait + Send + Sync,
    E: Send + Sync,
    Ty: EdgeType + Send + Sync {
    let neighbors = DashMap::<V, Vec<V>>::new();
    
    let nodes: Vec<V> = g.nodes().collect();

    nodes.par_iter().for_each(|&node|{
        let node_neighbors: Vec<V> = g.neighbors(node).collect();
        neighbors.insert(node, node_neighbors);
    });

    return neighbors;
}

/// Get the min neighbor of every node
pub fn get_vmins<V: NodeTrait + Send + Sync + Copy>(neighborhoods: &DashMap<V, Vec<V>>) -> DashMap<V, V>{
    let entries: Vec<_> = neighborhoods.iter().collect();

    /*let v_mins: DashMap<V, V> = entries.iter()
        .filter_map(|(&node, neighbors)|{
            neighbors.into_iter()
                .min()
                .map(|&v_min| (node, v_min))
        })
        .collect();
    */

    let v_mins: DashMap<V, V> = DashMap::new();

    entries.par_iter().for_each(|entry| {
        let (&key, vec) = entry.pair();
        if let Some(&min_value) = vec.iter().min() {
            let min_value = key.min(min_value);
            v_mins.insert(key, min_value);
        }
    });

    return v_mins;
}

//DEPRECATED
//TODO: generalize edges
pub fn min_selection_base<N>(g: &UnGraphMap<N, ()>) -> DiGraphMap<N, ()> 
    where N: NodeTrait + Eq + Send + Sync + Debug {
    let neighborhoods: DashMap<N, Vec<N>> = get_neighborhood(&g);
    let v_mins: DashMap<N, N> = get_vmins(&neighborhoods);


    // create directed graph h
    let mut h: DiGraphMap<N, ()> = DiGraphMap::new();
    
    //add edges
    for (u, neighbors) in neighborhoods{
        let v_min_option = v_mins.get(&u);
        
        if v_min_option.is_none(){
            continue;
        }
        
        let v_min = *v_min_option.unwrap();

        // base
        h.add_edge(u, v_min, ());
        for node in neighbors {
            //println!("[h] adding: {:?} -> {:?}", node, v_min);
            h.add_edge(node, v_min, ());
        }
    }

    return h;
}


/*
//w/o sorting of neighbors
// with Edge Pruning
pub fn min_selection<N>(g: &UnGraphMap<N, ()>) -> DiGraphMap<N, ()> 
    where N: NodeTrait + Eq + Send + Sync + Debug
{
    let neighborhoods: DashMap<N, Vec<N>> = get_neighborhood(&g);
    let v_mins: DashMap<N, N> = get_vmins(&neighborhoods);


    // create directed graph h
    let mut h: DiGraphMap<N, ()> = DiGraphMap::new();


    //add edges
    let mut nodes: HashSet<N> = g.nodes().collect();
    
    let mut neighborhoods_entries: Vec<_> = neighborhoods.iter().collect();
    neighborhoods_entries.sort_by(|a, b| a.key().cmp(b.key()));

    for entry in neighborhoods_entries{
        let &&n = &entry.key();
        let &neighbors = &entry.value();

        if !nodes.contains(&n){
            continue;
        }

        println!("MS: visiting node: {:?}", n);

        let n_min = *v_mins.get(&n).unwrap();
        
        //when a node is the minimum of its neighbourhood, it does not need to notify this information to its neighbours
        if n == n_min{
            for z in neighbors {
                let z_min = *v_mins.get(&z).unwrap();
                
                //when a node u is the local minimum in NN(u), [u = u_min] there are two exclusive cases
                if z_min == n_min{
                    h.add_edge(*z, n, ());
                }
                else{
                    h.add_edge(*z, z_min, ());
                    h.add_edge(n, z_min, ());
                }
                nodes.remove(&z);
            } 
        }
        else{
            //h.add_edge(n, n_min, ()); => get_neighborhood return <neighbors + node>
            for node in neighbors {
                //println!("adding: {:?} -> {:?}", node, v_min);
                h.add_edge(*node, n_min, ());
            }
        }

        nodes.remove(&n);
    }

    return h;
}
*/


// with Edge Pruning
pub fn min_selection_ep<N>(g: &UnGraphMap<N, ()>) -> DiGraphMap<N, ()> 
    where N: NodeTrait + Eq + Send + Sync + Debug {

    let neighborhoods: DashMap<N, Vec<N>> = get_neighborhood_base(&g);
    let v_mins: DashMap<N, N> = get_vmins(&neighborhoods);

    // create directed graph h
    let mut h: DiGraphMap<N, ()> = DiGraphMap::new();

    //add edges    
    let mut neighborhoods_entries: Vec<_> = neighborhoods.iter().collect();
    neighborhoods_entries.sort_by(|a, b| a.key().cmp(b.key()));

    for entry in neighborhoods_entries{
        let &&n = &entry.key();
        let &neighbors = &entry.value();

        let n_min_opt = v_mins.get(&n);
        if n_min_opt.is_none() {
            continue;
        }
        let n_min = *n_min_opt.unwrap();
        
        //when a node is the minimum of its neighbourhood, it does not need to notify this information to its neighbours
        if n == n_min{
            for z in neighbors {
                let z_min = *v_mins.get(&z).unwrap();
                
                //when a node u is the local minimum in NN(u), [u = u_min] there are two exclusive cases
                if z_min == n{
                    h.add_edge(*z, n, ());
                    //println!("[caso A] adding edge {:?}->{:?}", *z, n);
                }
                else{
                    h.add_edge(*z, z_min, ());
                    //println!("[caso B] adding edge {:?}->{:?}", *z, z_min);

                    h.add_edge(n, z_min, ());
                    //println!("[caso B] adding edge {:?}->{:?}", n, z_min);
                }

                //println!("removing {:?}", &z);
            } 
        }
        else{    
            h.add_edge(n, n_min, ());   // => get_neighborhood return <neighbors + node>
            //println!("[caso C] adding edge {:?}->{:?}", n, n_min);
            for node in neighbors {
                //println!("adding: {:?} -> {:?}", node, v_min);
                h.add_edge(*node, n_min, ());
                //println!("[caso C] adding edge {:?}->{:?}", *node, n_min);

            }
        }
    }
    return h;
}




//DEPRECATED
fn get_outgoing_neighborhood_seq<N: NodeTrait + Send + Sync>(h: &DiGraphMap<N, ()>) -> DashMap<N, Vec<N>>{
    let outgoing_neighborhoods: DashMap<N, Vec<N>> = DashMap::new();
    
    for n in h.nodes(){
        //outgoing_neighbour = {v | (u->v) € H}
        let mut local_outgoing = Vec::<N>::new();

        for dest in h.neighbors_directed(n, Outgoing){
            local_outgoing.push(dest);
        }

        outgoing_neighborhoods.insert(n, local_outgoing);
    }

    return outgoing_neighborhoods;
}


fn get_outgoing_neighborhood<N: NodeTrait + Send + Sync>(h: &DiGraphMap<N, ()>) -> DashMap<N, Vec<N>>{
    let outgoing_neighborhoods: DashMap<N, Vec<N>> = DashMap::new();
    
    /*for n in h.nodes(){
        //outgoing_neighbour = {v | (u->v) € H}
        let mut local_outgoing = Vec::<N>::new();

        for dest in h.neighbors_directed(n, Outgoing){
            local_outgoing.push(dest);
        }

        outgoing_neighborhoods.insert(n, local_outgoing);
    }*/

    let nodes: Vec<_> = h.nodes().collect();
    nodes.par_iter().for_each(|&n| {
        let mut local_outgoing: Vec<N> = Vec::new();

        for dest in h.neighbors_directed(n, Outgoing){
            local_outgoing.push(dest);
        }

        outgoing_neighborhoods.insert(n, local_outgoing);
    });

    return outgoing_neighborhoods;
}



pub fn prune<N: NodeTrait + Send + Sync + Copy + Debug>(h: DiGraphMap<N, ()>, tree: DiGraphMap<N, ()>) -> (UnGraphMap<N, ()>, DiGraphMap<N, ()>) {
    //println!("Pruning");
    //get outgoing neighborhoods
    let outgoing_neighborhoods: DashMap<N, Vec<N>> = get_outgoing_neighborhood(&h);

    let min_outgoing_neighborhoods = get_vmins(&outgoing_neighborhoods);

    let pruned_graph = UnGraphMap::<N, ()>
        ::with_capacity(h.node_count(), h.edge_count());
    
    /*
    no need to add node to pruned_graph
    when par_iterating, every node will be visited => every node will be added
    */

    //add to G(t+1) + deactivation
    let deactivated_nodes_mutex: Mutex<Vec<N>> = Mutex::new(Vec::new()); 
    let entries: Vec<_> = outgoing_neighborhoods.iter().collect();
    let pruned_graph_mutex = Mutex::new(pruned_graph);

    let tree_mutex = Mutex::new(tree);

    entries.par_iter().for_each(|entry|{
        let (u, neighbors) = entry.pair();

        if neighbors.len() > 1 {
            let v_min = *min_outgoing_neighborhoods.get(&u).unwrap();
            
            for v in neighbors{
                if *v != v_min{
                    pruned_graph_mutex.lock().unwrap()
                        .add_edge(*v, v_min, ());
                    //println!("[g]: adding edge {:?} -> {:?}", *v, v_min);
                }
            }
        }
        
        //deactivate nodes 
        if !neighbors.contains(u) {
            let v_min_opt = min_outgoing_neighborhoods.get(&u);
            //println!("v_min_opt: {:?}", v_min_opt);
            if v_min_opt.is_none(){
                //println!("min_outgoing_neighborhoods: do not found u");
                return;
            }

            let v_min = *v_min_opt.unwrap();
            tree_mutex.lock().unwrap()
                .add_edge(v_min, *u, ());

            //println!("Adding to tree: {:?} -> {:?}", v_min, *u);
            deactivated_nodes_mutex.lock().unwrap()
                .push(*u);
        }

        //TODO: 3rd case (node is seed: still active but NN(u) = {u})
        /*if (neighbors.len() == 1) && neighbors.contains(u) {
            deactivated_nodes_mutex.lock().unwrap()
                .push(*u);
        }*/
    });

    let deactivated_nodes = deactivated_nodes_mutex.into_inner()
        .unwrap_or(Vec::new());
    //deactivated_nodes.sort_unstable_by(|a, b| b.cmp(a));    //sort + reverse

    let mut pruned_graph = pruned_graph_mutex.into_inner().unwrap();
    let tree = tree_mutex.into_inner().unwrap();

    for deactivated in deactivated_nodes{
        //println!("Removing node: {:?}", deactivated);
        pruned_graph.remove_node(deactivated);
    }

    return (pruned_graph, tree);
}


pub fn prune_os<N: NodeTrait + Send + Sync + Copy + Debug>(h: DiGraphMap<N, ()>, tree: DiGraphMap<N, ()>) -> (DiGraphMap<N, ()>, DiGraphMap<N, ()>) {
    //get outgoing neighborhoods
    let outgoing_neighborhoods: DashMap<N, Vec<N>> = get_outgoing_neighborhood(&h);

    let min_outgoing_neighborhoods = get_vmins(&outgoing_neighborhoods);

    let pruned_graph = DiGraphMap::<N, ()>
        ::with_capacity(h.node_count(), h.edge_count());

    //add to G(t+1) + deactivation
    let deactivated_nodes_mutex: Mutex<Vec<N>> = Mutex::new(Vec::new()); 
    let entries: Vec<_> = outgoing_neighborhoods.iter().collect();
    let pruned_graph_mutex = Mutex::new(pruned_graph);

    let tree_mutex = Mutex::new(tree);

    entries.par_iter().for_each(|entry|{
        let (u, neighbors) = entry.pair();

        if neighbors.len() > 1 {
            let v_min = *min_outgoing_neighborhoods.get(&u).unwrap();
            
            for v in neighbors{
                if *v != v_min{
                    pruned_graph_mutex.lock().unwrap()
                        .add_edge(*v, v_min, ());
                }
            }
        }
        
        //deactivate nodes 
        if !neighbors.contains(u) {
            let v_min_opt = min_outgoing_neighborhoods.get(&u);
            if v_min_opt.is_none(){
                return;
            }

            let v_min = *v_min_opt.unwrap();
            tree_mutex.lock().unwrap()
                .add_edge(v_min, *u, ());

            deactivated_nodes_mutex.lock().unwrap()
                .push(*u);
        }

        //TODO: 3rd case (node is seed: still active but NN(u) = {u})
        /*if (neighbors.len() == 1) && neighbors.contains(u) {
            deactivated_nodes_mutex.lock().unwrap()
                .push(*u);
        }*/
    });

    let mut deactivated_nodes = deactivated_nodes_mutex.into_inner().unwrap_or(Vec::new());
    deactivated_nodes.sort_unstable_by(|a, b| b.cmp(a));    //sort + reverse

    let mut pruned_graph = pruned_graph_mutex.into_inner().unwrap();
    let tree = tree_mutex.into_inner().unwrap();

    //println!("pruned_graph: {:?}", pruned_graph);

    for deactivated in deactivated_nodes{
        //println!("Removing node: {:?}", deactivated);
        pruned_graph.remove_node(deactivated);
    }

    return (pruned_graph, tree);
}

pub fn seed_propagation<V: NodeTrait + Debug>(tree: DiGraphMap<V, ()>) -> HashMap<V, V>{
    let mut res: HashMap<V, V> = HashMap::new();

    let mut nodes: Vec<V> = tree.nodes().collect();
    //assert_eq!(nodes.len(), tree.node_count());
    //println!("Nodes: {:?}", nodes);
    nodes.sort_unstable();  //no duplicates => can use unstable sorting => more efficient

    while nodes.len() != 0 {    
        let min_node = nodes[0];        //sorted nodes => min node will always be the 1st
        let incoming_edge = tree.edges_directed(min_node, Incoming);    //either 0 or 1 edge
        //println!("{:?}", incoming_edge);

        for edge in incoming_edge{
            //println!("Node {:?}, edge {:?}", min_node, edge);

            if res.contains_key(&edge.0){
                let parent_seed = res.get(&edge.0).unwrap();
                res.insert(min_node, *parent_seed);
            }
            else{
                res.insert(min_node, edge.0);
            }
        }

        //no incoming edge into node => node is root of a tree
        if res.contains_key(&min_node) == false{
            res.insert(min_node, min_node);
        }

        nodes.remove(0);
    }

    return res;
}
