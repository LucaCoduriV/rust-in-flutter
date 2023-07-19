> 感谢您的帮助！非英语语言的文档可能会有语法不太通顺的问题。如果您想要为文档的改进做出贡献，请在 [Pull request](https://github.com/cunarist/rust-in-flutter/pulls) 留下您的意见。我们随时欢迎您的帮助，再次感谢！

# 🆎 Rust-In-Flutter

快速集成Rust代码到您的Flutter项目当中！  

![preview](https://github.com/cunarist/rust-in-flutter/assets/66480156/be85cf04-2240-497f-8d0d-803c40536d8e)  

兼具易用性、可拓展性和强悍性能的轻量级框架，在幕后为您默默处理所有问题！只需要在项目依赖中加入这个库，就可以开始编写您的Rust代码！  


## 优势

- 可集成任意数量的crates  
- 可以原样使用现有的crate包  
- 无需烦心于CMake、Gradle、Podfile等繁琐的构建文件  
- 开发过程中没有复杂的代码生成  
- 定义数量无限制的RESTful API  
- 通过简单的Dart请求和Rust响应实现异步交互  
- 从Rust到Dart的Stream  
- 在Dart项目热重载时重启Rust逻辑  
- 极小的性能开销  
- 发送native数据时没有memory copy  

## 平台支持

所有构建工作会被自动化完成。注意，Flutter项目中的文件不会受到影响。  

- ✅ Linux: 支持，已完成测试  
- ✅ Android: 支持，已完成测试  
- ✅ Windows: 支持，已完成测试  
- ✅ macOS: 支持，已完成测试  
- ✅ iOS: 支持，已完成测试  
- ⏸️ Web: 暂不支持，但正在[积极筹划](https://github.com/cunarist/rust-in-flutter/issues/34)  

> 若您有任何建议或者发现了bug，可以提交一份[issue](https://github.com/cunarist/rust-in-flutter/issues)或[pull](https://github.com/cunarist/rust-in-flutter/pulls)请求，我们会尽快回应您！

## 为什么使用Rust？

Dart确实是一个超棒的、现代化的、面向对象的编程语言。但由于需要垃圾回收等机制，它在性能上并不拔尖。在部分较为苛刻的场景下，我们可以考虑使用Rust，获得[2~40倍](https://programming-language-benchmarks.vercel.app/dart-vs-rust)的性能提升(甚至无需使用多线程)。  

# 👜 安装组件

首先，将rust_in_flutter添加到项目依赖：  

```bash
flutter pub add rust_in_flutter
```

然后安装Rust工具链。请参阅[Rust官方文档](https://doc.rust-lang.org/book/ch01-01-installation.html)。  

最后，检查系统环境是否已准备好进行编译。您可以在每个安装步骤后重复这些命令来验证系统状态。如果输出结果中没有问题，就可以开始了！  

```bash
rustc --version
flutter doctor
```

> 在Rust中使用各种不同的构建目标时，也许会遇到问题。不管遇到任何情况，您可以随时到[讨论页](https://github.com/cunarist/rust-in-flutter/discussions)发起一个Q&A来寻求帮助！  

# 👜 应用模板  

只需在 Flutter 项目文件夹下的命令行中运行以下命令即可：  

```bash
dart run rust_in_flutter:apply_template
```

运行命令后，会出现一些新的文件和文件夹，它们将成为Rust项目的初始模板。  

```diff
    my_flutter_project/
    ├── android/
    ├── ios/
    ├── lib/
*   │   ├── main.dart
    │   └── ...
    ├── linux/
+   ├── native/
+   │   ├── hub/
+   │   │   ├── src/
+   │   │   └── Cargo.toml
+   │   ├── sample_crate/
+   │   │   ├── src/
+   │   │   └── Cargo.toml
+   │   └── README.md
    ├── web/
    ├── windows/
*   ├── .gitignore
+   ├── Cargo.toml
*   ├── pubspec.yaml
    └── ...
```

别忘了先阅读`./native/README.md`文件。此外，您可能需要在生产环境中移除`sample_crate`。如果您已经有了要在这里使用的Rust crate，只需把它放在`./native`目录中，并将其设置为 `hub` crate 的一个依赖。  

现在请前往 `./native/hub/src/lib.rs`，您可以开始编写Rust代码了！  

# 🧱 注意事项

向Dart发起请求时，您应当指定operation和address。这种通信方式遵循RESTful API的标准。  

```dart
import 'package:msgpack_dart/msgpack_dart.dart';
import 'package:rust_in_flutter/rust_in_flutter.dart';

void someFunction() async {
    var rustRequest = RustRequest(
      address: 'basicCategory.counterNumber',
      operation: RustOperation.Read,
      bytes: serialize(
        {
          'letter': 'Hello from Dart!',
          'before_number': 888,
          'dummy_one': 1,
          'dummy_two': 2,
          'dummy_three': [3, 4, 5]
        },
      ),
    );

    var rustResponse = await requestToRust(rustRequest);
    var message = deserialize(rustResponse.bytes) as Map;
    var innerValue = message['after_number'] as int;
}
```

在 Rust 中接收到请求后，应当先按address对其进行分类：  

```rust
pub async fn handle_request(request_unique: RustRequestUnique) -> RustResponseUnique {
    let rust_request = request_unique.request;
    let interaction_id = request_unique.id;

    let layered: Vec<&str> = rust_request.address.split('.').collect();
    let rust_response = if layered.is_empty() {
        RustResponse::default()
    } else if layered[0] == "basicCategory" {
        if layered.len() == 1 {
            RustResponse::default()
        } else if layered[1] == "counterNumber" {
            sample_functions::calculate_something(rust_request).await
        } else {
            RustResponse::default()
        }
    } else {
        RustResponse::default()
    };

    RustResponseUnique {
        id: interaction_id,
        response: rust_response,
    }
}
```

Rust代码中的Endpoint函数应该如下图所示：    
> 请在match中定义消息模式(Message Schema)，因为不同类型会有不同的消息模式。  

```rust
pub async fn calculate_something(rust_request: RustRequest) -> RustResponse {
    match rust_request.operation {
        RustOperation::Create => RustResponse::default(),
        RustOperation::Read => {
            #[allow(dead_code)]
            #[derive(Deserialize)]
            struct RustRequestSchema {
                letter: String,
                before_number: i32,
                dummy_one: i32,
                dummy_two: i32,
                dummy_three: Vec<i32>,
            }
            let slice = rust_request.bytes.as_slice();
            let received: RustRequestSchema = from_slice(slice).unwrap();
            println!("{:?}", received.letter);

            let before_value = received.before_number;
            let after_value = sample_crate::add_seven(before_value);

            #[derive(Serialize)]
            struct RustResponseSchema {
                after_number: i32,
                dummy_one: i32,
                dummy_two: i32,
                dummy_three: Vec<i32>,
            }
            RustResponse {
                successful: true,
                bytes: to_vec_named(&RustResponseSchema {
                    after_number: after_value,
                    dummy_one: 1,
                    dummy_two: 2,
                    dummy_three: vec![3, 4, 5],
                })
                .unwrap(),
            }
        }
        RustOperation::Update => RustResponse::default(),
        RustOperation::Delete => RustResponse::default(),
    }
}
```

您可以扩展这种RESTful API模式，并根据需要创建成百上千个endpoint。如果您有web开发背景，这可能会让您觉得很熟悉。更多注释和细节包含在Rust模板的代码中，供您参阅。  

理想情况下，**Flutter**处理跨平台用户界面，而**Rust**负责业务逻辑。前端和后端可以完全分离，这意味着Dart和Rust代码各司其职。这两个世界通过channel和stream进行通信。  

我们使用[MessagePack](https://msgpack.org/)来序列化Dart和Rust之间发送的消息(正如Rust模板所提供的那样)，除非您有其他理由不这么做。MessagePack是一种嵌套的二进制结构，类似于JSON，但速度更快、体积更小。  

在Dart和Rust之间传递数据基本上都是字节数组(bytes array)，Dart中称之为 `Uint8List`，而Rust中称之为`Vec<u8>`。虽然我们推荐使用MessagePack进行序列化，但您也可以发送任何类型的字节数据，例如高分辨率图像或某种文件数据。若您不需要发送额外的数据信息，可以直接发送一个空的字节数组。  

# ☕ 支持我们  

😉 如果您从Rust-In-Flutter的功能中受益，并认为它对您非常有帮助，为什么不考虑下支持这个项目呢？您的慷慨捐助将有助于Rust-In-Flutter项目的维护和开发，确保其不断改进、发展！  

若有此想法，您可以[打赏一下](https://www.buymeacoffee.com/cunarist)我们。  