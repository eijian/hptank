// findsame: find same images in the tank
//
// Usage: cat <input data> | findsame [<skip count>]
//

use std::collections::BTreeMap;
use std::collections::HashSet;
use std::env;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::str;
use std::sync::{Arc, Mutex};

use hex;
use kdtree::distance::squared_euclidean;
use kdtree::KdTree;
use rayon::prelude::*;

type Color = [f32; 3];

pub const COLOR0: Color = [0.0, 0.0, 0.0];
pub const DIFF: i32 = 30;  // 12% of 255
pub const TOLERANCE: i32 = 30 * 64;
pub const SDEVLIM: i32 = 100;        // 色ずれの分散

#[derive(PartialEq)]
enum Status {
  Filed,
  Deleted,
  Discarded,
  Pending
}

impl Status {
  fn to_str(&self) -> &str {
    match self {
      Status::Filed => "filed",
      Status::Deleted => "deleted",
      Status::Discarded => "discarded",
      Status::Pending => "pending"
    }
  }
}

impl fmt::Display for Status {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.to_str())
  }
}

struct Image {
  id: u64,
  reso: (u32, u32),
  size: u32,
  fp: Vec<u8>,
  color: Color,
  status: Status,
  check: bool
}

impl Image {
  pub const RADIUS: f32 = 0.0015;
}

type ColTree = KdTree<f32, u64, Color>;

fn main() {
  let args: Vec<String> = env::args().collect();
  let skip: u64 = if args.len() == 2 {
    match args[1].parse::<u64>() {
      Ok(n) => n,
      _     => 0
    }
  } else {
    0
  };

  let (mut images, tree) = read_imagedata();
  eprintln!("#IMAGE: {}", images.len());

  // cf. https://rustforbeginners.hatenablog.com/entry/arc-mutex-design-pattern
  let keys: Vec<_> = images.keys().cloned().collect();
  let checked = Arc::new(Mutex::new(HashSet::new()));
  images.par_iter().for_each(|(id, im)| {
    if *id >= skip && im.status != Status::Deleted &&
       !checked.lock().unwrap().contains(id) {
      let mut ids: Vec<u64> = vec![];
      for i in near_images(&id, &images, &tree)
                 .iter()
                 .filter(|&i| images.get(i).unwrap().status != Status::Deleted) {
        ids.push(*i);
        checked.lock().unwrap().insert(*i);
      }

      if ids.len() > 0 {
        print_image(&id, &images);
        ids.iter().for_each(|i| {
          print_image(&i, &images);
        });
        println!("")
      }
    }
  });

}

fn print_image(id: &u64, images: &BTreeMap<u64, Image>) {
  let img = images.get(id).unwrap();
  print!("{}({:?},{},({:.0},{:.0},{:.0}))/", id, img.reso, img.size,
    img.color[0] * 1000f32, img.color[1] * 1000f32, img.color[2] * 1000f32);
}

fn read_imagedata() -> (BTreeMap<u64, Image>, ColTree) {
  let mut images = BTreeMap::new();
  let mut tree: ColTree = ColTree::new(3);

  let stdin = io::stdin();
  for l in stdin.lock().lines() {
    if let Ok(im) = l {
      let image = to_image(&im);
      if image.status == Status::Discarded {
        continue
      };
      let id = image.id;
      tree.add(image.color, id).unwrap();
      images.insert(id, image);
    }
  };
  (images, tree)
}

/*
fn build_trees(images: &BTreeMap<u64, Image>) -> [ColTree; 4] {
  let mut trees: [ColTree; 4] = [ColTree::new(3), ColTree::new(3), ColTree::new(3), ColTree::new(3)];
  for (id, im) in images.iter() {
    let colmap = im.colmap;
    trees[0].add(colmap[0].to_arr(), *id).unwrap();
    trees[1].add(colmap[1].to_arr(), *id).unwrap();
    trees[2].add(colmap[2].to_arr(), *id).unwrap();
    trees[3].add(colmap[3].to_arr(), *id).unwrap();
  };
  trees
}

#[test]
fn test_build_trees() {
  let l1 = "1|eaepc-000456821173836a4ea02d8b900fb4b4877ba7d2.jpg|aea7a88c82816c60608b7f7d897d7a645e5d817f80a29c9cb0a8aaa0979a6b61665f555a72686981797b9f989caca5aaa69c9da9a0a57a7179554c54796d73a3999fafa6aea8a0a8968a87a1979c80767e584e5780747ea79da8a399a48e858c948a849a8d8f8b7d866c5c65877b859e94a2817985605960948e8685797883757c72626c7a6d79776e7c5e586456535b7976755a5559564e58554d5958505c625b6679757c9392956f6f775a596453505a5b57616a677089888ea09f9fa6a4a0|20160117090937||filed|2000|1333|356147|0";
  let l2 = "2|eaepc-000456821173836a4ea02d8b900fb4b4877ba7d2.jpg|ffffffffffffffffffffffff897d7a645e5d817f80a29c9cb0a8aaa0979a6b61665f555a72686981797b9f989caca5aaa69c9da9a0a57a7179554c54796d73a3999fafa6aea8a0a8968a87a1979c80767e584e5780747ea79da8a399a48e858c948a849a8d8f8b7d866c5c65877b859e94a2817985605960948e8685797883757c72626c7a6d79776e7c5e586456535b7976755a5559564e58554d5958505c625b6679757c9392956f6f775a596453505a5b57616a677089888ea09f9fa6a4a0|20160117090937||filed|2000|1333|356147|0";
  let l3 = "3|eaepc-000456821173836a4ea02d8b900fb4b4877ba7d2.jpg|ffa7a88c82816c60608b7f7d897d7a645e5d817f80a29c9cb0a8aaa0979a6b61665f555a72686981797b9f989caca5aaa69c9da9a0a57a7179554c54796d73a3999fafa6aea8a0a8968a87a1979c80767e584e5780747ea79da8a399a48e858c948a849a8d8f8b7d866c5c65877b859e94a2817985605960948e8685797883757c72626c7a6d79776e7c5e586456535b7976755a5559564e58554d5958505c625b6679757c9392956f6f775a596453505a5b57616a677089888ea09f9fa6a4a0|20160117090937||filed|2000|1333|356147|0";
  let l4 = "114|eaepc-04d9c472dc53072c2fb8adaf8da28fadf033b122.jpg|d8a486d39e80a8775c744d3967422d6f412593582da26331dba688d5a081ae7b5e9c6b52845741582f1a7e47239b5e2ed39c7ad39b7ad39b7ad2997ab582678e5f468e593591562a936e56a37961daa587f0bc9de9b698c1917583594054331c4b44416d5a52c1937deaba9df1c0a1c0917473503c2e211a423f3e695751916d5dc39982f3c2a3e1af8fa2785d4b372a3d393868544c634c42957563e2b095e7b498cc9a7d845e482e26235e483f4b39326f584aa27e6ba5806ca47d687b5744|20160117090938||filed|460|697|41722|0";
  let l5 = "910|eaepc-1c46b796128920955203f6a3e395a7b0a047aa46.jpg|b29895b58979b87e63ba8266b98268b27a609e654d7c4b3cb38270b7826ebf846bbe856cbb846ab27a609a614c6e3e32b17257b87962c88f79c18b75b9836caa715a8e57465a2c23a56c59b6806fc58e7dbf8674bb85729f6452723d30491f158d503c955746a06354a16557ae736390564760342a502a1f8852408e513e965c4d8c5f599b685ea36b5c9d6f609065567255569261529f6b5b9874739e7871b78776cb9884be8c7896776bab7c63a06d5a9a767294726eb38779cf9e89c1907b|20160117090944||filed|1440|810|125183|0";

  let mut map = BTreeMap::new();
  let im1 = to_image(&l1);
  map.insert(im1.id, im1);
  let im2 = to_image(&l2);
  map.insert(im2.id, im2);
  let im3 = to_image(&l3);
  map.insert(im3.id, im3);
  let im4 = to_image(&l4);
  map.insert(im4.id, im4);
  let im5 = to_image(&l5);
  map.insert(im5.id, im5);
  let trees = build_trees(&map);

  let ls0 = near_image_list(&0, &map.get(&1).unwrap().colmap[0], &trees[0]);
  assert_eq!(ls0, vec![1, 3]);
  let ls1 = near_image_list(&0, &map.get(&1).unwrap().colmap[1], &trees[1]);
  assert_eq!(ls1, vec![1, 2, 3]);
  assert_eq!(and_array(&ls0, &ls1), vec![1, 3]);
  let ls20 = near_image_list(&0, &map.get(&114).unwrap().colmap[0], &trees[0]);
  assert_eq!(ls20, vec![114, 910]);
}
*/

fn near_images(id: &u64, images: &BTreeMap<u64, Image>, tree: &ColTree) -> Vec<u64> {
  //let mut imageids: Vec<u64> = vec![];
  let srcimg = images.get(id).unwrap();
  let id0 = near_image_list(id, &srcimg.color, &tree);
  if id0.len() == 0 {
    vec![]
  } else {
    let mut nears: Vec<u64> = vec![];
    let srcfp = &srcimg.fp;
    for i in id0.iter() {
      if (*i as i64 - *id as i64).abs() == 1 {  // 連続した同じような写真が含まれないように
          continue
      }
      let dstfp = &images.get(i).unwrap().fp;
      if same(srcfp, dstfp) {
        nears.push(*i)
      }
    }
    nears
  }
}

fn same(srcfp: &Vec<u8>, dstfp: &Vec<u8>) -> bool {
  if srcfp.len() == 0 || dstfp.len() == 0 {
    return false
  }
  let mut judge = true;
  let mut totaldiff = 0;
  let mut sdev: i32 = 0;

  for (a, b) in srcfp.iter().zip(dstfp.iter()) {
    let diff = (*a as i32 - *b as i32).abs();
    totaldiff += diff;
    if diff > DIFF || totaldiff > TOLERANCE {
      judge = false;
      break
    }
    sdev += diff * diff;
  }
  if judge == true {
    if sdev / (srcfp.len() as i32) > SDEVLIM {
      judge = false
    }
  }
  judge
}

#[test]
fn test_same() {
  let fp1 = "8d7451917955937b59856c546d584660584852514443443a9582649e896aa1896e9f7c699173687e858668787e5055559a8c77a89079ac8b76af8575a68a83a0b0ba8ba5b5666f779c8e7eb08a75b68572bb8c7fb28a82a39592929a9b7d858895897baa8572b17b65c28b78bb897da98a849f9d9d959a9f74746f8a796ea17360b87c69b67f70a27b75908d9289919a42494e5f5a5b936e698d5f588c5f59976d6e77707d58657719202e4f46518d73814d414f43364281616c7a63763a3c55";
}

fn near_image_list(id: &u64, color: &Color, tree: &ColTree) -> Vec<u64> {
  let mut id0 = if let Ok(res) = tree.within(color, Image::RADIUS, &squared_euclidean) {
    let (_, ids): (Vec<f32>, Vec<u64>) = res.into_iter()
                                            .filter(|(_, x)| *x != id)
                                            .unzip();
    //ids.iter().filter(|x| *x != id).collect()
    ids
  } else {
    vec![]
  };
  id0.sort();
  id0
}

fn and_array(id1: &Vec<u64>, id2: &Vec<u64>) -> Vec<u64> {
  if id1.len() == 0 {
    return vec![]
  };
  if id2.len() == 0 {
    return vec![]
  };
  let mut id2iter = id2.iter();
  let mut ids: Vec<u64> = vec![];
  let mut i2: &u64 = if let Some(i) = id2iter.next() {
    i
  } else {
    &0
  };
  'comploop:
  for i1 in id1 {
    //if *i1 == *i2 {
    //  ids.push(*i1)
    //} else if *i1 < *i2 {
    if *i1 < *i2 {
      continue
    };
    while *i1 > *i2 {
       i2 = match id2iter.next() {
         None => &0,
         Some(i) => i
       };
       if *i2 == 0 {
         break 'comploop
       }
    };
    if *i1 == *i2 {
      ids.push(*i1)
    };
  };
  ids
}

#[test]
fn test_and_array() {
  let a1 = vec![1, 2, 5, 9, 10, 11, 12];
  let a2 = vec![2, 6, 10, 12, 15];
  assert_eq!(and_array(&a1, &a2), vec![2, 10, 12]);
  let a3 = vec![1, 2, 3];
  let a4 = vec![1, 3];
  assert_eq!(and_array(&a3, &a4), vec![1, 3]);
  let a5 = vec![1, 3];
  let a6 = vec![1, 2, 3];
  assert_eq!(and_array(&a5, &a6), vec![1, 3]);
}

fn to_image(line: &str) -> Image {
  let im: Vec<&str> = line.split('|').collect();
  let (f, c) = to_color(im[2]);
  Image {
    id: im[0].parse().unwrap(),
    reso: (im[6].parse().unwrap(), im[7].parse().unwrap()),
    size: im[8].parse().unwrap(),
    fp: f,
    color: c,
    status: to_status(im[5]),
    check: false
  }
}

fn to_color(fp: &str) -> (Vec<u8>, Color) {
  let hfp = hex::decode(fp).unwrap();

  let mut r: i16 = 0;
  let mut g: i16 = 0;
  let mut b: i16 = 0;
  for c in hfp.chunks(3) {
    r += c[0] as i16;
    g += c[1] as i16;
    b += c[2] as i16;
  };
  let mag: f32 = 255.0 * 64.0;
  (hfp, [r as f32 / mag, g as f32 / mag, b as f32 / mag])
}

#[test]
fn test_to_color() {
  //let fp = "ac9b87a08979896e5f76553f7a66468f996b91a8669eb25b8d664678573f6c4c3b7d564275594a978f86afb99cb6c17985613f7e6652674a3e774d3e8b6961b7a6a3cec6b8c7c49bb1937bb19e978e6f6872433b8e6058b48f7ec3a390c9b49bc9aea3caafa6b79080996657966459b38874c39d85cbb09cb99d96bd9b8ec19982ba907ac09b86cca991c5a083ba9d84b0938cac8877b3886fa9816abb9b86d9beaadbbea7c5a487b79e99a58478a17966926957a27e6bd2b49ee5cdbae0c7b3";
  let fp = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
  let (hfp, color) = to_color(&fp);
  assert_eq!(color, [0.5, 0.5, 0.5]);
  let fp2 = "ffffffffffffffffffffffff7f7f007f7f007f7f007f7f00ffffffffffffffffffffffff7f7f007f7f007f7f007f7f00ffffffffffffffffffffffff7f7f007f7f007f7f007f7f00ffffffffffffffffffffffff7f7f007f7f007f7f007f7f00007f7f007f7f007f7f007f7f000000000000000000000000007f7f007f7f007f7f007f7f000000000000000000000000007f7f007f7f007f7f007f7f000000000000000000000000007f7f007f7f007f7f007f7f000000000000000000000000";
  let (hfp2, color2) = to_color(&fp2);
  assert_eq!(color2, [0.3745098, 0.4990196, 0.3745098]);
  let fp3 = "d8a486d39e80a8775c744d3967422d6f412593582da26331dba688d5a081ae7b5e9c6b52845741582f1a7e47239b5e2ed39c7ad39b7ad39b7ad2997ab582678e5f468e593591562a936e56a37961daa587f0bc9de9b698c1917583594054331c4b44416d5a52c1937deaba9df1c0a1c0917473503c2e211a423f3e695751916d5dc39982f3c2a3e1af8fa2785d4b372a3d393868544c634c42957563e2b095e7b498cc9a7d845e482e26235e483f4b39326f584aa27e6ba5806ca47d687b5744";
  let (hfp3, color3) = to_color(&fp3);
  assert_eq!(color3, [0.6082108, 0.4483456, 0.3550858]);
  let fp4 = "b29895b58979b87e63ba8266b98268b27a609e654d7c4b3cb38270b7826ebf846bbe856cbb846ab27a609a614c6e3e32b17257b87962c88f79c18b75b9836caa715a8e57465a2c23a56c59b6806fc58e7dbf8674bb85729f6452723d30491f158d503c955746a06354a16557ae736390564760342a502a1f8852408e513e965c4d8c5f599b685ea36b5c9d6f609065567255569261529f6b5b9874739e7871b78776cb9884be8c7896776bab7c63a06d5a9a767294726eb38779cf9e89c1907b";
  let (hfp4, color4) = to_color(&fp4);
  assert_eq!(color4, [0.6319853, 0.43186274, 0.365625]);

  // id = 7811
  let fp5 = "8d7451917955937b59856c546d584660584852514443443a9582649e896aa1896e9f7c699173687e858668787e5055559a8c77a89079ac8b76af8575a68a83a0b0ba8ba5b5666f779c8e7eb08a75b68572bb8c7fb28a82a39592929a9b7d858895897baa8572b17b65c28b78bb897da98a849f9d9d959a9f74746f8a796ea17360b87c69b67f70a27b75908d9289919a42494e5f5a5b936e698d5f588c5f59976d6e77707d58657719202e4f46518d73814d414f43364281616c7a63763a3c55";
  let (hfp5, color5) = to_color(&fp5);
  assert_eq!(color5, [0.5376226, 0.46286765, 0.43621323]);
  // id = 420670
  let fp6 = "8d7550927955947b59866d546d584660584852514443443a9582659e8a6ba18a6e9f7c6a9173687e868669797f5055559b8d78a99179ac8c77af8575a78a83a1b0bb8ca6b66770779d8e7eb18b76b68572bb8d80b38a82a49692939a9b7e858996897caa8572b27b66c38b79bc8a7da98a85a09d9e969ba07574708a7a6ea17460b97d69b67f71a27b75918e938a929a42494e5f5a5b946e698d5f588d5f5a986e6e77707e586577181f2e4f45528d73814d404f42364281616d7b6477393c55";
  let (hfp6, color6) = to_color(&fp6);
  assert_eq!(color6, [0.53927696, 0.46409315, 0.43768382]);

}

fn to_status(s: &str) -> Status {
  match s {
    "filed"     => Status::Filed,
    "deleted"   => Status::Deleted,
    "discarded" => Status::Discarded,
    "pending"   => Status::Pending,
    _           => Status::Pending
  }
}

//
