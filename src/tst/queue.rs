/// Copyright (c) Migrant Coder, 2015
/// All rights reserved.

extern crate mru;

use std::mem;
use std::os;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

use mru::lockfree::{ Queue, QueueConsumer, QueueProducer };

fn current_thread_name() -> String {
  match thread::current().name() {
    Some(name) => name.to_string(),
    None => "unknown".to_string()
  }
}

fn produce(
    element_count: usize,
    id_offset: usize,
    queue: QueueProducer<usize>) {
  println!("{} - produce from ID {}", current_thread_name(), id_offset);

  let end_id = id_offset + element_count;
  for id in id_offset..end_id {
    queue.push(id);
  }

  println!(
      "{} - produced to ID {}",
      current_thread_name(),
      id_offset + element_count - 1);
}

fn consume(
    element_count: usize,
    queue: QueueConsumer<usize>,
    consumed: &mut Vec<u64>) {
  println!("{} - consume", current_thread_name());

  let mut n = 0;
  while n < element_count {
    match queue.pop() {
      Some(id) => {
        consumed[id] = 1;
        n = n + 1;
      },
      None => {}
    }
  }

  println!("{} - consumed {}", current_thread_name(), n);
}

fn test_concurrent_produce_consume(
    producer_count: usize,
    consumer_count: usize,
    element_count: usize,
    iteration_count: usize) {
  // The queue instance to test.
  let queue: Queue<usize> = Queue::new(128);
  let arc_queue = Arc::new(queue);
  let queue_producer = QueueProducer::new(&arc_queue);
  let queue_consumer = QueueConsumer::new(&arc_queue);

  let elements_per_producer = element_count/producer_count;
  let elements_per_consumer = element_count/consumer_count;

  for _ in 0..iteration_count {
    let mut thread_id = 0;

    // Track consumed IDs.
    let mut consumed: Vec<u64> = Vec::with_capacity(element_count);
    for _ in 0..element_count {
      consumed.push(0);
    }
    let shared_consumed = Arc::new(consumed);

    // Produce.
    let mut producers: Vec<JoinHandle> = Vec::new();
    for i in 0..producer_count {
      let id_offset = elements_per_producer * i;
      let sp = queue_producer.clone();
      producers.push(
          thread::Builder::new().
              name(format!("thread {}", thread_id)).
              spawn(move || { produce(elements_per_producer, id_offset, sp); }).
              unwrap());
      thread_id += 1;
    }

    // Consume.
    let mut consumers: Vec<JoinHandle> = Vec::new();
    for _ in 0..consumer_count {
      let consumed = shared_consumed.clone();
      let sc = queue_consumer.clone();
      consumers.push(
          thread::Builder::new().
              name(format!("thread {}", thread_id)).
              spawn(move || {
                // Unsynchronized access OK on x86-64 due to element type width.
                let c: &mut Vec<u64> = unsafe {mem::transmute(&*consumed)};
                consume(elements_per_consumer, sc, c)
              }).
              unwrap());
      thread_id += 1;
    }

    // Wait.
    for p in producers {
      let _ = p.join();
    }
    for c in consumers {
      let _ = c.join();
    }

    // Verify.
    // assert!(arc_queue.is_empty());
    let mut found_unconsumed = false;
    for i in 0..element_count {
      if shared_consumed[i] == 0 {
        found_unconsumed = true;
      }
    }
    if found_unconsumed {
      println!("stopping");
      break;
    }
  }
}

fn usage(program: &String) -> String {
  format!("usage: {} PRODUCERS CONSUMERS ELEMENTS [ITERATIONS]", program)
}

fn main() {
  let args: Vec<String> = os::args();
  if args.len() < 4 {
    println!("{}", usage(&args[0]));
    return;
  }

  let producer_count: usize = FromStr::from_str(&args[1]).unwrap();
  let consumer_count: usize = FromStr::from_str(&args[2]).unwrap();
  let element_count: usize = FromStr::from_str(&args[3]).unwrap();
  let iteration_count: usize =
      if args.len() > 4 {
        FromStr::from_str(&args[4]).unwrap()
      } else {
        1
      };
  if producer_count > element_count {
        println!("PRODUCERS must be <= ELEMENTS")
  }
  if consumer_count > element_count {
        println!("CONSUMERS must be <= ELEMENTS")
  }

  test_concurrent_produce_consume(
      producer_count,
      consumer_count,
      element_count,
      iteration_count);
}
