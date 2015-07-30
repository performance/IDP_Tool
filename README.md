Tool to measure Open and Short Bad Bump Bonds from IDP images.
``` text
USAGE:
	idp_tool.exe [FLAGS] [OPTIONS] --test_dir < test_directory >

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --ignore_edges < ignore_edges >          number of rows/cols to ignore along the edges.
    -o, --open_threshold < open_threshold >      Threshold to use for open test.
    -s, --short_threshold < short_threshold >    % of the median to use as threshold for short test.
    -t, --test_dir < test_directory >            Test area with each sub dir containing idp images.

```
With this tool, The user passes in a test directory and obtains defect counts for images

idp_tool -t < test dir > -o < open Threshold > 

The structure of the test directory is 
``` text
  	test\
  	test\x1y1\
  	test\x1y1\blahblah_C1717_blahblah_PNResetOut_blah.idp
  	test\x1y1\blahblah_C1717_blahblah_PNSignalOut_blah.idp
  	test\x1y1\blahblah_Cv1v2_blahblah_PNResetOut_blah.idp
  	...
  	The file names are of the format *_Cv1v2_*_PNResetOut_*.idp
  	All images have a dead band. For now this is hardcoded in the tool.
```
* This tool collects test images by directory and filters out files that do not have PNResetOut in the name
* Then it arranges the four *PNRestOut*.idp files into two pairs, by v1 == v2 and v1 != v2
* The pair with v1 == v2 is considered for open test.
  * Unknown open pixels are All pixels in the image with the absolute difference of the imges in the open pair;  
  * Open bad pixels are all the unknown pixels with value less than the open threshold [ passed in as a cmd line arg ]
* The pair with v1 != v2 is considered for short test.
  * masked pixels are All the pixel locations from the open bad test that are in the dead band, or open bad, or on columns or rows that have > 50% open bad pixels
  * unknown short pixels are all the pixels in the short absoule difference that are not on the masked locations
  * Short bad pixels are All the unknown short pixels with value less than 75% of the medain of unknown short pixels

Expects a final output to stdout as a csv file 
short_bad_diagonal_pairs: adjacent diagonals that have more than 50% bad short pixels.

``` csv

test_no, case x, case y,  #open_bad_pixels, open_threshold, number_of_open_bads_in_bad_cols, number_of_open_bads_in_bad_rows, #open_bad_cols, #open_bad_rows, #short_bad_pixels, short_threshold, number_of_short_bads_not_in_bad_diagonals, number_of_bad_diagonals, number_of_adjacent_bad_diagonals, #measured_pixels
0, 11, 11, 1286563, 0.5, 15983, 5967, 18, 6, 485760, 0.22023636, 6111, 411, 22, 2598544
1, 11, 12, 580196, 0.5, 826, 0, 1, 0, 0, 0.38518336, 0, 0, 0, 2598544
2, 12, 5, 2034947, 0.5, 2034947, 1730675, 1844, 1202, 0, 0, 0, 0, 0, 2598544
3, 15, 11, 1593095, 0.5, 1152980, 900652, 1272, 809, 0, 0.36080655, 0, 0, 0, 2598544
```

To redirect the output to a csv file, use 
``` Bash
    idp_tool.exe -t test -o 0.3 -i 10 -s 0.75 > asdfg 2>&1
```