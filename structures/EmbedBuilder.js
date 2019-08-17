class RichEmbed {
    constructor() {
        this.title;
        this.description;
        this.url;
        this.timestamp;
        this.color;
        this.footer;
        this.image;
        this.thumbnail;
        this.video;
        this.provider;
        this.author;
        this.fields = [];
    }

    setTitle(title) {
        this.title = title;
        return this;
    }

    setDescription(description) {
        this.description = description;
        return this;
    }

    setURL(url) {
        this.url = url;
        return this;
    }

    setTimestamp(timestamp) {
        this.timestamp = timestamp.toISOString();
        return this;
    }

    setColor(color) {
        if (typeof color === "string") color = parseInt(color, 16);
        this.color = color;
        return this;
    }

    setFooter(text, icon) {
        this.footer = {text: text, icon_url: icon};
        return this;
    }

    setImage(url) {
        this.image = {url: url};
        return this;
    }

    setThumbnail(url) {
        this.thumbnail = {url: url};
        return this;
    }

    setVideo(url) {
        this.video = {url};
        return this;
    }

    setProvider(name, url) {
        this.video = {name, url};
        return this;
    } 

    setAuthor(name, url, icon_url) {
        this.author = {name, url, icon_url};
        return this;
    }

    addField(name, value, inline = false) {
        this.fields.push({name, value, inline});
        return this;
    }

    render() {
        return {
            title: this.title,
            type: "rich",
            description: this.description,
            url: this.url,
            timestamp: this.timestamp,
            color: this.color != null ? this.color : 16765448,
            footer: this.footer,
            image: this.image,
            thumbnail: this.thumbnail,
            video: this.video,
            provider: this.provider,
            author: this.author,
            fields: this.fields
        };
    }
}

module.exports = RichEmbed;