# Список доступных команд по умолчанию
default:
    @just --list

# === Генерация и сборка Rust ===

# Сгенерировать Kotlin-типы данных
codegen:
    cd Android && cargo run --package shared --bin codegen --features codegen,facet_typegen -- --language kotlin --output-dir generated

# Собрать нативную библиотеку через boltffi (с поддержкой 16 KB страниц)
pack-android:
    cd shared && boltffi pack android

# Полный цикл обновления Rust-слоя (генерация типов + сборка .so)
build-rust: codegen pack-android

# === Сборка и запуск Android ===

# Собрать APK
build-android: build-rust
    cd Android && ./gradlew assembleDebug

# Установить приложение на подключенное устройство
install: build-android
    cd Android && ./gradlew installDebug

# Запустить приложение на устройстве
start:
    adb shell am start -n com.crux.examples.counter/.MainActivity

# Полный цикл: собрать всё, установить и запустить одной командой
run: install start

# Остановить запущенное приложение
stop:
    adb shell am force-stop com.crux.examples.counter

# Смотреть логи приложения
logs:
    adb logcat -s "MainActivity" "Crux" "AndroidRuntime"

# === Очистка проекта ===

# Очистить кэш Rust и Gradle
clean:
    cd shared && cargo clean
    cd Android && ./gradlew clean
    rm -rf Android/generated/jniLibs

# Запустить десктопное приложение
run-desktop:
    cargo run -p desktop
