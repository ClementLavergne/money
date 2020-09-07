import { AccountClient } from "money";

// Singleton
const account = new AccountClient()
// Tags management
const inputTag = document.getElementById("input-tag");
const tagsList = document.getElementById("tag-list");
const addTag = document.getElementById("add-tag");
const removeTag = document.getElementById("remove-tag");
// Resources management
const inputResource = document.getElementById("input-resource");
const resourcesList = document.getElementById("resource-list");
const addResource = document.getElementById("add-resource");
const removeResource = document.getElementById("remove-resource");
// Orders
const pre = document.getElementById("money-canvas");
const addOrder = document.getElementById("add-order");
const ordersTable = document.getElementById("orders");
// File management
const loadData = document.getElementById("load-data");
const downloadData = document.getElementById("download-data");
// Sum
const sum = document.getElementById("sum-canvas");

const refreshList = (node, items) => {
    // Remove existing
    while (node.firstChild) {
        node.removeChild(node.lastChild);
    }

    // Update
    items.forEach(function(item){
        var option = document.createElement('option');
        option.value = item;
        node.appendChild(option);
    });
};

const addOrderRow = (obj) => {
    var row = ordersTable.insertRow();

    // Date
    var date = document.createElement("input");
    date.type = "text";
    date.value = obj.order.date;
    date.addEventListener('keyup', ({key}) => {
        if (key === "Enter") {
            if (account.set_order_date(obj.id, date.value.toString())) {
                console.log("Order " + obj.id + " date: " + date.value.toString())
                requestAnimationFrame(render);
            }
        }
    })
    row.insertCell().appendChild(date);

    // Description node
    var description = document.createElement("input");
    description.type = "text";
    description.value = obj.order.description;
    description.addEventListener('keyup', ({key}) => {
        if (key === "Enter") {
            if (account.set_order_description(obj.id, description.value.toString())) {
                console.log("Order " + obj.id + " description: " + description.value.toString())
                requestAnimationFrame(render);
            }
        }
    })
    row.insertCell().appendChild(description);

    // Amount node
    var amount = document.createElement("input");
    amount.type = "text";
    amount.value = obj.order.amount.toFixed(2) + "€";
    amount.addEventListener('keyup', ({key}) => {
        if (key === "Enter") {
            if (amount.value == "") {
                amount.value = "0.0"
            }

            const float = parseFloat(amount.value);
            if (account.set_order_amount(obj.id, float)) {
                console.log("Order " + obj.id + " amount: " + float.toFixed(2))
                requestAnimationFrame(render);
            }
        }
    })
    amount.addEventListener('click', event => {
        amount.value = ""
    });
    row.insertCell().appendChild(amount);

    // Resource node
    var resource = document.createElement("select");
    var empty_option = document.createElement("option");
    empty_option.value = "-";
    empty_option.text = "-";
    empty_option.disabled = true
    resource.appendChild(empty_option);
    account.resources().forEach(function(item) {
        var option = document.createElement("option");
        option.value = item;
        option.text = item;
        resource.appendChild(option);
    });
    if (obj.order.resource != null) {
        resource.value = obj.order.resource
    } else {
        resource.value = "-"
    }
    resource.addEventListener('change', function() {
        if (account.set_order_resource(obj.id, this.value)) {
            console.log("Order " + obj.id + " resource: " + this.value)
            requestAnimationFrame(render);
        }
    }, false)
    row.insertCell().appendChild(resource);

    // Tags
    var tags = document.createElement("select")
    tags.multiple = true
    account.tags().forEach(function(item) {
        var option = document.createElement("option");
        option.value = item;
        option.text = item;

        obj.order.tags.forEach(function(tag) {
            if (item == tag) {
                option.selected = true
            }
        })

        tags.appendChild(option);
    });
    tags.addEventListener('change', function() {
        const selectedValues = [...this.options]
                     .filter((x) => x.selected)
                     .map((x)=>x.value)

        if (account.set_order_tags(obj.id, selectedValues)) {
            console.log("Order " + obj.id + " tags: " + selectedValues)
            requestAnimationFrame(render);
        }
    }, false)
    row.insertCell().appendChild(tags);

    // State
    var state = document.createElement("select");
    const options = ["Pending", "InProgress", "Done"]
    var id = 0
    for (const text of options) {
        var option = document.createElement("option");
        option.text = text
        option.value = text
        id++
        state.appendChild(option)
    }
    state.value = obj.order.state;
    state.addEventListener('change', function() {
        if (account.set_order_state(obj.id, this.value)) {
            console.log("Order " + obj.id + " state: " + this.value)
            requestAnimationFrame(render);
        }
    }, false)
    row.insertCell().appendChild(state);

    // Remove button node
    var remove_button = document.createElement("input");
    remove_button.type = "button";
    remove_button.value = "remove";
    remove_button.addEventListener('click', event => {
        if (account.delete_order(obj.id)) {
            console.log("Order " + obj.id + ": removed!")
        }
        requestAnimationFrame(render);
    });
    row.insertCell().appendChild(remove_button);
}

addTag.addEventListener("click", event => {
    account.add_tag(inputTag.value.toString())
    refreshList(tagsList, account.tags())
    inputTag.value = "";
    requestAnimationFrame(render);
});

removeTag.addEventListener("click", event => {
    account.remove_tag(inputTag.value)
    refreshList(tagsList, account.tags())
    inputTag.value = "";
    requestAnimationFrame(render);
});

addResource.addEventListener("click", event => {
    account.add_resource(inputResource.value.toString())
    refreshList(resourcesList, account.resources())
    inputResource.value = "";
    requestAnimationFrame(render);
});

removeResource.addEventListener("click", event => {
    account.remove_resource(inputResource.value)
    refreshList(resourcesList, account.resources())
    inputResource.value = "";
    requestAnimationFrame(render);
});

addOrder.addEventListener("click", event => {
    account.add_order();
    requestAnimationFrame(render);
});

// Load YAML file
loadData.addEventListener("change", function() {
    var file = this.files[0];
    var reader = new FileReader();

    reader.readAsText(file,'UTF-8');

    reader.onload = readerEvent => {
        var content = readerEvent.target.result;

        if (account.load(content)) {
            console.log("File '" + file + "' loaded!")
            requestAnimationFrame(render);
        }
    }
}, false);

// Write YAML file
downloadData.addEventListener("click", event => {
    function download(filename, text) {
        var element = document.createElement('a');
        element.setAttribute('href', 'data:text/plain;charset=utf-8,' + encodeURIComponent(text));
        element.setAttribute('download', filename);

        element.style.display = 'none';
        document.body.appendChild(element);

        element.click();

        document.body.removeChild(element);
    }

    const filename = prompt("Please enter file name:", "account-data.yml")

    if (filename != null) {
        const data = account.serialize_account_yaml()
        download(filename, data);
    }
});

const render = () => {
    console.log("Render!")
    refreshList(tagsList, account.tags())
    refreshList(resourcesList, account.resources())

    // Clear table rows
    for (var i=ordersTable.rows.length-1; i >=1; i--) {
        ordersTable.deleteRow(i);
    }

    if (account.orders().length == 0) {
        pre.textContent = "No orders";
    } else {
        // Add table rows
        var text = "";
        account.orders().forEach(function(item) {
            var obj = JSON.parse(item);
            addOrderRow(obj);

            text += item + "\n\r";
        });

        pre.textContent = text;
    }

    // Sum
    sum.textContent = account.sum_orders().toFixed(2) + '€'
};

requestAnimationFrame(render);
