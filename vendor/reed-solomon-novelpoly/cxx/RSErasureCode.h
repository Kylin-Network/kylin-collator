/*
Encoding/erasure decoding for Reed-Solomon codes over binary extension fields
Author: Sian-Jheng Lin (King Abdullah University of Science and Technology (KAUST), email: sianjheng.lin@kaust.edu.sa)

This program is the implementation of
Lin, Han and Chung, "Novel Polynomial Basis and Its Application to Reed-Solomon Erasure Codes," FOCS14.
(http://arxiv.org/abs/1404.3458)
*/


typedef unsigned short GFSymbol;
typedef int Boolean;

#define FIELD_BITS 16

extern GFSymbol mask;
extern GFSymbol Base[FIELD_BITS];

#define FIELD_SIZE (1<<FIELD_BITS)//Field size
#define MODULO (FIELD_SIZE-1)

extern GFSymbol LOG_TABLE[FIELD_SIZE];
extern GFSymbol EXP_TABLE[FIELD_SIZE];

//-----Used in decoding procedure-------
extern GFSymbol skewVec[MODULO];//twisted factors used in FFT
extern GFSymbol B[FIELD_SIZE>>1];//factors used in formal derivative
extern GFSymbol log_walsh[FIELD_SIZE];//factors used in the evaluation of the error locator polynomial

//return a*EXP_TABLE[b] over GF(2^r)
GFSymbol mulE(GFSymbol a, GFSymbol b);

void walsh(GFSymbol* data, int size);

void formal_derivative(GFSymbol* cos, int size);

void IFLT(GFSymbol* data, int size, int index);
void FLT(GFSymbol* data, int size, int index);

void setup();

//Encoding alg for k/n<0.5: message is a power of two
void encodeL(GFSymbol* data, int k, GFSymbol* codeword, int n);
void decode_init(Boolean* erasure, GFSymbol* log_walsh2, int n);

void decode_main(GFSymbol* codeword, int k, Boolean* erasure, GFSymbol* log_walsh2, int n);

int roundtrip(int n, int k);
int test_flt_roundtrip();
