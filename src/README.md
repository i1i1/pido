# Report (Ryibin Ivan 2020 Group 5) 

As an algorithm for the assignment, in this project is used some simple but yet
tweaked algorithm. This algorithm can be splitted into 2 algorithms:

1. Poligonization
2. Colorization

## Poligonization

The point of this part is to divide image in triangles. This algorithm starts
with certain amount of triangles in random places on 512 x 512 image and them randomly
permutates a part of them fit all of them on image.

As fitness function here is used some kind of this function:
``` rust
fn fitness(im: Image, polygons: Vec<Polygon>) {
    let mut fitness = 0;

    for pix in im.pixels() {
        let delta = {
            for pol in polygons {
                if pix.is_inside(pol) {
                    break 0;
                }
            }
            1
        };

        fitnes += delta;
    }
}
```

It counts number of pixels outside of polygons and so the algorithm tries to minimize those by permutating.
 

## Colorization

The point of this part is to colorize triangles from previous stage. This
algorithm starts with random colors randomly permutates colors of different
polygons. a part of them fit all of them on image.

As fitness function here is used some kind of L2 loss function:
``` rust
fn fitness(im: Image, refimg: Image) {
    let mut fitness = 0;

    for (Color(imr, img, imb), Color(refr, refg, refb)) in im.pixels().zip(refimg.pixels()) {
        let dr = refr - imr;
        let dg = refg - img;
        let db = refb - imb;
        fitnes += ((dr * dr + dg * dg + db * db) as f64).sqrt();
    }
}
```
 
