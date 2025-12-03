#![allow(non_snake_case)]

use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::{jlong, jint, jfloat};

use crate::{game_init, game_resize, game_update, game_render, game_set_direction, game_set_mode, game_touch, game_destroy, GameHandle};

#[no_mangle]
pub extern "system" fn Java_com_example_flutter_1con_GameNative_gameInit(
    _env: JNIEnv,
    _class: JClass,
    width: jint,
    height: jint,
) -> jlong {
    let handle = game_init(width as u32, height as u32);
    handle as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_example_flutter_1con_GameNative_gameResize(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    width: jint,
    height: jint,
) {
    game_resize(handle as GameHandle, width as u32, height as u32);
}

#[no_mangle]
pub extern "system" fn Java_com_example_flutter_1con_GameNative_gameUpdate(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    game_update(handle as GameHandle);
}

#[no_mangle]
pub extern "system" fn Java_com_example_flutter_1con_GameNative_gameRender(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    game_render(handle as GameHandle);
}

#[no_mangle]
pub extern "system" fn Java_com_example_flutter_1con_GameNative_gameSetDirection(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    direction: jint,
) {
    game_set_direction(handle as GameHandle, direction);
}

#[no_mangle]
pub extern "system" fn Java_com_example_flutter_1con_GameNative_gameSetMode(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    mode: jint,
) {
    game_set_mode(handle as GameHandle, mode);
}

#[no_mangle]
pub extern "system" fn Java_com_example_flutter_1con_GameNative_gameTouch(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    x: jfloat,
    y: jfloat,
    action: jint,
) {
    game_touch(handle as GameHandle, x, y, action);
}

#[no_mangle]
pub extern "system" fn Java_com_example_flutter_1con_GameNative_gameDestroy(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    game_destroy(handle as GameHandle);
}
