import {
    Account,
    delete_account_order,
    TransactionState,
    set_account_order_date,
    set_account_order_description,
    set_account_order_amount,
    set_account_order_resource,
    set_account_order_tags,
    set_account_order_state,
    get_account_resources,
    get_account_orders,
    get_account_tags,
    load_account_data,
    serialize_account_as_yaml
} from "money";

// Singleton
const account = new Account()
// Tags management
const inputTag = document.getElementById("input-tag")
const tagsList = document.getElementById("tag-list")
const addTag = document.getElementById("add-tag")
const removeTag = document.getElementById("remove-tag")
// Resources management
const inputResource = document.getElementById("input-resource")
const resourcesList = document.getElementById("resource-list")
const addResource = document.getElementById("add-resource")
const removeResource = document.getElementById("remove-resource")
// Orders
const pre = document.getElementById("money-canvas")
const addOrder = document.getElementById("add-order")
const ordersTable = document.getElementById("orders")
// File management
const loadData = document.getElementById("load-data")
const downloadData = document.getElementById("download-data")
// Sum
const sum = document.getElementById("sum-canvas")

const refreshList = (node, items) => {
    // Remove existing
    while (node.firstChild) {
        node.removeChild(node.lastChild)
    }

    // Update
    items.forEach(function(item){
        var option = document.createElement('option')
        option.value = item
        node.appendChild(option)
    });
};

const addOrderRow = (obj) => {
    var row = ordersTable.insertRow()

    // Date
    var date = document.createElement("input")
    date.type = "text"
    date.value = obj.order.date
    date.addEventListener('keyup', ({key}) => {
        if (key === "Enter") {
            if (set_account_order_date(account, obj.id, date.value)) {
                console.log("Order " + obj.id + " date: " + date.value)
                requestAnimationFrame(render)
            } else {
                alert("Expected date format:  YY-MM-DD")
            }
        }
    })
    row.insertCell().appendChild(date)

    // Description node
    var description = document.createElement("input")
    description.type = "text"
    description.value = obj.order.description
    description.addEventListener('keyup', ({key}) => {
        if (key === "Enter") {
            if (set_account_order_description(account, obj.id, description.value)) {
                console.log("Order " + obj.id + " description: " + description.value)
                requestAnimationFrame(render)
            }
        }
    })
    row.insertCell().appendChild(description)

    // Amount node
    var amount = document.createElement("input")
    amount.type = "text"
    amount.value = obj.order.amount.toFixed(2) + "€"
    amount.addEventListener('keyup', ({key}) => {
        if (key === "Enter") {
            if (amount.value == "") {
                amount.value = "0.0"
            }

            const float = parseFloat(amount.value)
            if (set_account_order_amount(account, obj.id, float)) {
                console.log("Order " + obj.id + " amount: " + float.toFixed(2))
                requestAnimationFrame(render)
            }
        }
    })
    amount.addEventListener('click', event => {
        amount.value = ""
    })
    row.insertCell().appendChild(amount)

    // Resource node
    var resource = document.createElement("select")
    var empty_option = document.createElement("option")
    empty_option.value = "-"
    empty_option.text = "-"
    empty_option.disabled = true
    resource.appendChild(empty_option)
    get_account_resources(account).forEach(function(item) {
        var option = document.createElement("option")
        option.value = item
        option.text = item
        resource.appendChild(option)
    });
    if (obj.order.resource != null) {
        resource.value = obj.order.resource
    } else {
        resource.value = "-"
    }
    resource.addEventListener('change', function() {
        if (set_account_order_resource(account, obj.id, this.value)) {
            console.log("Order " + obj.id + " resource: " + this.value)
            requestAnimationFrame(render)
        }
    }, false)
    row.insertCell().appendChild(resource)

    // Tags
    var tags = document.createElement("select")
    tags.multiple = true
    get_account_tags(account).forEach(function(item) {
        var option = document.createElement("option")
        option.value = item
        option.text = item

        obj.order.tags.forEach(function(tag) {
            if (item == tag) {
                option.selected = true
            }
        })

        tags.appendChild(option)
    });
    tags.addEventListener('change', function() {
        const selectedValues = [...this.options]
                     .filter((x) => x.selected)
                     .map((x)=>x.value)

        if (set_account_order_tags(account, obj.id, selectedValues)) {
            console.log("Order " + obj.id + " tags: " + selectedValues)
            requestAnimationFrame(render)
        }
    }, false)
    row.insertCell().appendChild(tags)

    // State
    var state = document.createElement("select")
    const nb_elements = Object.entries(TransactionState).length
    const entries = Object.entries(TransactionState).slice((nb_elements / 2), nb_elements)
    var id = 0
    for (const entry of entries) {
        var option = document.createElement("option")
        option.text = entry[0]
        option.value = entry[0]
        id++
        state.appendChild(option)
    }
    state.value = obj.order.state;
    state.addEventListener('change', function() {
        // Find corresponding index
        var index = 0
        for (const entry of entries) {
            if (this.value == entry[0]) {
                index = entry[1]
                break
            }
        }
        if (set_account_order_state(account, obj.id, index)) {
            console.log("Order " + obj.id + " state: " + this.value)
            requestAnimationFrame(render)
        }
    }, false)
    row.insertCell().appendChild(state)

    // Remove button node
    var remove_button = document.createElement("input")
    remove_button.type = "button"
    remove_button.value = "remove"
    remove_button.addEventListener('click', event => {
        if (delete_account_order(account, obj.id)) {
            console.log("Order " + obj.id + ": removed!")
        }
        requestAnimationFrame(render)
    });
    row.insertCell().appendChild(remove_button)
}

addTag.addEventListener("click", event => {
    if (account.add_tag(inputTag.value) == undefined) {
        refreshList(tagsList, account.tags())
        inputTag.value = ""
        requestAnimationFrame(render)
    }
});

removeTag.addEventListener("click", event => {
    if (account.remove_tag(inputTag.value) == undefined) {
        refreshList(tagsList, account.tags())
        inputTag.value = ""
        requestAnimationFrame(render)
    }
});

addResource.addEventListener("click", event => {
    if (account.add_resource(inputResource.value) == undefined) {
        refreshList(resourcesList, account.resources())
        inputResource.value = ""
        requestAnimationFrame(render)
    }
});

removeResource.addEventListener("click", event => {
    if (account.remove_resource(inputResource.value) == undefined) {
        refreshList(resourcesList, account.resources())
        inputResource.value = ""
        requestAnimationFrame(render)
    }
});

addOrder.addEventListener("click", event => {
    account.add_order()
    requestAnimationFrame(render)
});

// Load YAML file
loadData.addEventListener("change", function() {
    var file = this.files[0]
    var reader = new FileReader()

    reader.readAsText(file,'UTF-8')

    reader.onload = readerEvent => {
        var content = readerEvent.target.result;

        if (load_account_data(account, content)) {
            console.log("File '" + file + "' loaded!")
            requestAnimationFrame(render);
        }
    }
}, false);

// Write YAML file
downloadData.addEventListener("click", event => {
    function download(filename, text) {
        var element = document.createElement('a')
        element.setAttribute('href', 'data:text/plain;charset=utf-8,' + encodeURIComponent(text));
        element.setAttribute('download', filename)

        element.style.display = 'none'
        document.body.appendChild(element)

        element.click()

        document.body.removeChild(element)
    }

    const filename = prompt("Please enter file name:", "account-data.yml")

    if (filename != null) {
        const data = serialize_account_as_yaml(account)
        download(filename, data)
    }
});

const render = () => {
    console.log("Render!")
    refreshList(tagsList, get_account_tags(account))
    refreshList(resourcesList, get_account_resources(account))

    // Clear table rows
    for (var i=ordersTable.rows.length-1; i >=1; i--) {
        ordersTable.deleteRow(i)
    }

    const orders = get_account_orders(account)
    if (orders.length == 0) {
        pre.textContent = "No orders"
    } else {
        // Add table rows
        var text = ""
        orders.forEach(function(item) {
            var obj = JSON.parse(item)
            addOrderRow(obj)

            text += item + "\n\r"
        })

        pre.textContent = text
    }

    // Sum
    sum.textContent = account.sum_orders().toFixed(2) + '€'
};

requestAnimationFrame(render)
