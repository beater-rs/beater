(function() {var implementors = {};
implementors["bytes"] = [{"text":"impl&lt;B:&nbsp;<a class=\"trait\" href=\"bytes/buf/trait.Buf.html\" title=\"trait bytes::buf::Buf\">Buf</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/io/trait.BufRead.html\" title=\"trait std::io::BufRead\">BufRead</a> for <a class=\"struct\" href=\"bytes/buf/struct.Reader.html\" title=\"struct bytes::buf::Reader\">Reader</a>&lt;B&gt;","synthetic":false,"types":["bytes::buf::reader::Reader"]}];
implementors["futures_util"] = [{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/io/trait.BufRead.html\" title=\"trait std::io::BufRead\">BufRead</a> for <a class=\"struct\" href=\"futures_util/io/struct.AllowStdIo.html\" title=\"struct futures_util::io::AllowStdIo\">AllowStdIo</a>&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/io/trait.BufRead.html\" title=\"trait std::io::BufRead\">BufRead</a>,&nbsp;</span>","synthetic":false,"types":["futures_util::io::allow_std::AllowStdIo"]}];
implementors["protobuf"] = [{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/std/io/trait.BufRead.html\" title=\"trait std::io::BufRead\">BufRead</a> for <a class=\"struct\" href=\"protobuf/struct.CodedInputStream.html\" title=\"struct protobuf::CodedInputStream\">CodedInputStream</a>&lt;'a&gt;","synthetic":false,"types":["protobuf::coded_input_stream::CodedInputStream"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()