trait Message : Send {}

struct MessageWrapper {
	num: Box<Message:Send>:Send
}

struct SenderWrapper {
	sender: Sender<MessageWrapper>
}

struct SenderSender {
	sender: Sender<SenderWrapper>
}

fn main() {
	let (send, recv) = channel::<MessageWrapper>();
	let s = SenderWrapper { sender: send };
	let (_send, _recv) = channel::<SenderWrapper>();
	let _s = SenderSender { sender: _send };
	_s.sender.send(s);
}
