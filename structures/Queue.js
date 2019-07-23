class Queue {
    constructor(client, executor) {
        this.internalQueue = [];
        this.client = client;
        this.executor = executor;
    }

    enqueue(param) {
        this.internalQueue.push(param);
        this.process();
    }

    async process() {
        if (this.internalQueue.length === 0) {
            if (this.processing) {
                this.processing = false;
            }
        }

        if (this.processing === true) return;

        this.processing = true;

        let tempQueue = this.internalQueue;
        for (let i = 0; i < tempQueue.length; i++) {
            await this.executor(tempQueue[i]);
        }

        delete tempQueue;
        this.internalQueue = this.internalQueue.filter(param => !tempQueue.includes(param));
        this.processing = false;
    }

    clear() {
        this.internalQueue = [];
    }
}

module.exports = Queue;