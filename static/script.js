function getJson(url, cbk) {
    let xhr = new XMLHttpRequest();
    xhr.onreadystatechange = function () {
        if (cbk && this.readyState == 4 && this.status == 200) {
            cbk(JSON.parse(this.responseText));
        }
    };
    xhr.open("GET", url, true);
    xhr.send();
}

function putJson(url, data) {
    let xhr = new XMLHttpRequest();
    xhr.open("PUT", url, true);
    xhr.setRequestHeader('Content-type', 'application/json');
    xhr.send(JSON.stringify(data));
}

function postJson(url, data, cbk) {
    let xhr = new XMLHttpRequest();
    xhr.onreadystatechange = function () {
        if (cbk && this.readyState == 4) {
            if (this.status == 200) {
                cbk(JSON.parse(this.responseText));
            } else {
                try {
                    cbk(JSON.parse(this.responseText));
                } catch (e) {
                    cbk(this.responseText);
                }
            }
        }
    };
    xhr.open("POST", url, true);
    xhr.setRequestHeader('Content-type', 'application/json');
    xhr.send(JSON.stringify(data));
}

function convertToJson(form, cbk) {
    let formData = {};
    for (let i = 0; i < form.elements.length; i++) {
        let element = form.elements[i];
        if (element.type !== "submit" && element.name) {
            if (isNaN(element.value)) {
                formData[element.name] = element.value;
            } else {
                formData[element.name] = Number(element.value);
            }
        }
    }
    postJson(form.action, formData, cbk)
}
