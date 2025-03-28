#include <jni.h>
#include "react-native-zksync-sso.h"

extern "C"
JNIEXPORT jdouble JNICALL
Java_com_zksyncsso_ZksyncSsoModule_nativeMultiply(JNIEnv *env, jclass type, jdouble a, jdouble b) {
    return zksyncsso::multiply(a, b);
}
