(function() {var implementors = {};
implementors["io_lifetimes"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/std/os/fd/raw/trait.IntoRawFd.html\" title=\"trait std::os::fd::raw::IntoRawFd\">IntoRawFd</a> for <a class=\"struct\" href=\"io_lifetimes/struct.OwnedFd.html\" title=\"struct io_lifetimes::OwnedFd\">OwnedFd</a>","synthetic":false,"types":["io_lifetimes::types::OwnedFd"]}];
implementors["nix"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/std/os/fd/raw/trait.IntoRawFd.html\" title=\"trait std::os::fd::raw::IntoRawFd\">IntoRawFd</a> for <a class=\"struct\" href=\"nix/pty/struct.PtyMaster.html\" title=\"struct nix::pty::PtyMaster\">PtyMaster</a>","synthetic":false,"types":["nix::pty::PtyMaster"]}];
implementors["rustix"] = [{"text":"impl&lt;'context, T:&nbsp;<a class=\"trait\" href=\"rustix/fd/trait.AsFd.html\" title=\"trait rustix::fd::AsFd\">AsFd</a> + <a class=\"trait\" href=\"rustix/fd/trait.IntoFd.html\" title=\"trait rustix::fd::IntoFd\">IntoFd</a> + <a class=\"trait\" href=\"rustix/fd/trait.FromFd.html\" title=\"trait rustix::fd::FromFd\">FromFd</a>&gt; <a class=\"trait\" href=\"rustix/fd/trait.IntoRawFd.html\" title=\"trait rustix::fd::IntoRawFd\">IntoRawFd</a> for <a class=\"struct\" href=\"rustix/io/epoll/struct.Epoll.html\" title=\"struct rustix::io::epoll::Epoll\">Epoll</a>&lt;<a class=\"struct\" href=\"rustix/io/epoll/struct.Owning.html\" title=\"struct rustix::io::epoll::Owning\">Owning</a>&lt;'context, T&gt;&gt;","synthetic":false,"types":["rustix::imp::io::epoll::Epoll"]},{"text":"impl <a class=\"trait\" href=\"rustix/fd/trait.IntoRawFd.html\" title=\"trait rustix::fd::IntoRawFd\">IntoRawFd</a> for <a class=\"struct\" href=\"rustix/io/struct.OwnedFd.html\" title=\"struct rustix::io::OwnedFd\">OwnedFd</a>","synthetic":false,"types":["rustix::io::owned_fd::OwnedFd"]}];
implementors["same_file"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/std/os/fd/raw/trait.IntoRawFd.html\" title=\"trait std::os::fd::raw::IntoRawFd\">IntoRawFd</a> for <a class=\"struct\" href=\"same_file/struct.Handle.html\" title=\"struct same_file::Handle\">Handle</a>","synthetic":false,"types":["same_file::Handle"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()