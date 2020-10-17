var SITE_ROOT_PATH = "/vaalikoppi/";
var SOUND_STATE = 0;
var currentVotingId = -1;
var votesGiven = 0;

// Helper functions
function getCookie(name) {
  let cookieValue = null;
  if (document.cookie && document.cookie !== "") {
    const cookies = document.cookie.split(";");
    for (let i = 0; i < cookies.length; i++) {
      const cookie = cookies[i].trim();
      if (cookie.substring(0, name.length + 1) === name + "=") {
        cookieValue = decodeURIComponent(cookie.substring(name.length + 1));
        break;
      }
    }
  }
  return cookieValue;
}

function callApi(url, method, body) {
  return fetch(url, {
    method: method,
    headers: {
      "Content-Type": "application/json",
      "X-CSRFToken": getCookie("csrftoken"),
      "X-Requested-With": "XMLHttpRequest",
    },
    mode: "same-origin",
    body: JSON.stringify(body),
  });
}

function getVotingForm(votingId) {
  return document.getElementById(`voting-form-${votingId}`);
}

// Used to sort candidates by position in transferable election confirmation modal
function compareChosenCandidates(a, b) {
  if (a.position.charAt(0) == "-") {
    return 1;
  }
  if (b.position.charAt(0) == "-") {
    return -1;
  }
  if (a.position < b.position) {
    return -1;
  }
  if (a.position > b.position) {
    return 1;
  }
  return 0;
}

// Voting
function getChosenCandidates(isTransferable, votingId) {
  const form = getVotingForm(votingId);
  const chosenCandidates = Array.from(
    form.querySelectorAll(
      isTransferable ? ".voting-order" : "input[name=candidate]:checked"
    )
  ).map((input) => {
    const candidateId = input.value;
    const nameNode = form.querySelector(
      `label[for=candidate-v-${votingId}-${candidateId}]`
    );
    const labelText = nameNode.childNodes[0].textContent;
    const labelRankedPosition = isTransferable
      ? nameNode.previousElementSibling.innerHTML
      : -1;
    return { id: candidateId, name: labelText, position: labelRankedPosition };
  });
  return chosenCandidates;
}

function showVotingConfirmationModal(isTransferable, chosenCandidates) {
  const candidatesString = chosenCandidates
    .map((c) => (isTransferable ? `${c.position}.${c.name}` : c.name))
    .join(", ");
  const singularOrPlural =
    chosenCandidates.length > 1 ? "ehdokkaita:\n" : "ehdokasta:\n";
  const elem = document.querySelector("#voting-modal");
  const instance = M.Modal.getInstance(elem);
  document.getElementById(
    "voting-modal-text"
  ).innerHTML = `Olet äänestämässä ${singularOrPlural} ${candidatesString}`;
  instance.open();
}

function setVotingConfirmationEventListener(
  isTransferable,
  votingId,
  chosenCandidates
) {
  const votingArea = document.getElementById("voting-list-area");
  const form = getVotingForm(votingId);

  document
    .getElementById("voting-modal-confirm")
    .addEventListener("click", () => {
      Array.from(
        form.querySelectorAll("input[name=candidate]:checked")
      ).forEach((elem) => elem.setAttribute("disabled", true));

      const data = {
        candidates: isTransferable
          ? chosenCandidates.map((c) => `${c.id}:${c.position}`)
          : chosenCandidates.map((c) => c.id),
      };

      callApi(
        `${SITE_ROOT_PATH}votings/${votingId}/${
          isTransferable ? "vote-transferable" : "vote"
        }/`,
        "POST",
        data
      )
        .then((res) => res.text())
        .then((html) => (votingArea.innerHTML = html))
        .catch(() => {
          alert("Äänestäminen epäonnistui. Päivitä sivu ja yritä uudelleen!");
          refreshVotingList();
        });
    });
}

function vote(votingId) {
  const chosenCandidates = getChosenCandidates(false, votingId);
  showVotingConfirmationModal(false, chosenCandidates);
  setVotingConfirmationEventListener(false, votingId, chosenCandidates);
}

function voteTransferableElection(votingId) {
  const chosenCandidates = getChosenCandidates(true, votingId).sort(
    compareChosenCandidates
  );
  showVotingConfirmationModal(true, chosenCandidates);
  setVotingConfirmationEventListener(true, votingId, chosenCandidates);
}

async function refreshVotingList(admin = false) {
  const votingArea = document.getElementById("voting-list-area");
  const adminPath = admin ? "admin/" : "";

  return callApi(`${SITE_ROOT_PATH}${adminPath}votings/list/`, "GET")
    .then((res) => res.text())
    .then((html) => (votingArea.innerHTML = html))
    .catch(() => {
      alert(
        "Äänestysten haku ei onnistunut. Päivitä sivu. Jos koetit äänestää, katso, näkyykö äänestys jo äänestettynä."
      );
    });
}

function checkboxClick(votingId, candidateId) {
  const form = document.getElementById(`voting-form-${votingId}`);
  const maxVotes = parseInt(form.getAttribute("data-voting-max-votes"));
  const formCheckboxes = [...form.children].filter(
    (c) => c.type === "checkbox"
  );

  if (formCheckboxes.length > maxVotes) {
    formCheckboxes
      .filter(
        (c) => c.getAttribute("id") === `candidate-v${candidateId}-${votingId}`
      )
      .forEach((c) => (c.checked = true));
  }
}

// User login / logout
function logout() {
  callApi(`${SITE_ROOT_PATH}user/logout/`, "POST")
    .then((res) => res.json())
    .then((data) => {
      if (data.status === 0) {
        document.cookie = "";
        location.reload();
        localStorage.removeItem("token");
      }
    })
    .catch(() => alert("Uloskirjautuminen epäonnistui. Päivitä sivu."));
}

function submitToken() {
  var token = document.getElementById("type-token-field").value;
  var notificationArea = document.getElementById("login-notification-area");

  notificationArea.classList.add("loading-token-notification");
  notificationArea.classList.remove("wrong-token-warning");
  notificationArea.innerHTML = "Ladataan...";

  callApi(`${SITE_ROOT_PATH}user/login/`, "POST", { token })
    .then(() => location.reload())
    .catch(() =>
      window.setTimeout(() => {
        notificationArea.classList.remove("loading-token-notification");
        notificationArea.classList.add("wrong-token-warning");
        notificationArea.innerHTML = "Virheellinen koodi";
      }, 100)
    );

  localStorage.setItem("token", token);
}

// Admin
function generateTokens(count) {
  const generateTokensButton = document.getElementById(
    "generate-tokens-button"
  );
  generateTokensButton.setAttribute("disabled", true);

  callApi(`${SITE_ROOT_PATH}admin/tokens/generate/`, "POST", { count })
    .then(() => {
      alert("Koodien generointi onnistui.");
      location.reload();
    })
    .catch(() => alert("Koodien generointi epäonnistui."));
  generateTokensButton.setAttribute("disabled", false);
}

function activateOrInvalidateToken(isActivate, code, number) {
  var token = code;

  if (token.length < 1) {
    return;
  }

  var button = document.getElementById(
    `${isActivate ? "activate" : "invalidate"}-token-button-${number}`
  );
  var clickedState = button.dataset["clicked"];

  // Require two clicks to activate code
  if (clickedState == "0") {
    button.innerHTML = isActivate ? "Aktivoi?" : "Mitätöi?";
    button.classList.add("orange");
    button.classList.remove(isActivate ? "green" : "red");
    button.dataset["clicked"] = "1";
    return;
  }

  button.setAttribute("disabled", true);

  callApi(
    `${SITE_ROOT_PATH}admin/tokens/${isActivate ? "activate" : "invalidate"}/`,
    "POST",
    { token }
  )
    .then(() => location.reload())
    .catch(() =>
      alert(
        `Koodin ${
          isActivate ? "aktivointi" : "mitätöinti"
        } epäonnistui. Tarkista koodi.`
      )
    );
}

function createVoting() {
  const isTransferable = document.getElementById("is-transfer-election")
    .checked;
  const votingName = document.getElementById("voting-name").value;
  const votingDescription = document.getElementById("voting-description").value;
  const maxVotes = document.getElementById("max-votes").value;

  const data = {
    is_transferable: isTransferable,
    voting_name: votingName,
    voting_description: votingDescription,
    max_votes: maxVotes,
  };
  callApi(`${SITE_ROOT_PATH}admin/votings/create/`, "POST", data)
    .then(() => refreshVotingList(true))
    .catch(() =>
      alert("Äänestyksen luominen ei ehkä onnistunut! Päivitä sivu!")
    );
}

function addCandidate(votingId, isTransferable) {
  const candidate = document.getElementById(`voting-${votingId}-candidate-name`)
    .value;
  if (candidate) {
    const data = {
      is_transferable: isTransferable,
      candidate_name: candidate,
    };
    callApi(`${SITE_ROOT_PATH}admin/votings/${votingId}/add/`, "POST", data)
      .then(() => refreshVotingList(true))
      .catch(() =>
        alert("Ehdokkaan lisääminen ei ehkä onnistunut! Päivitä sivu!")
      );
  }
}

function removeCandidate(candidate_id, is_transferable) {
  const data = {
    is_transferable,
  };
  callApi(
    `${SITE_ROOT_PATH}admin/votings/${candidate_id}/remove/`,
    "POST",
    data
  )
    .then(() => refreshVotingList(true))
    .catch(() =>
      alert("Äänestyksen luominen ei ehkä onnistunut! Päivitä sivu!")
    );
}

function closeVoting(votingId, is_transferable) {
  const data = {
    is_transferable,
  };
  callApi(`${SITE_ROOT_PATH}admin/votings/${votingId}/close/`, "POST", data)
    .then(() => {
      refreshVotingList(true);

      // If a sound is already playing, reveal the result with a badum-tss sound
      if (SOUND_STATE !== 0) {
        playSound(3);
      }

      if (data.not_voted_tokens && data.not_voted_tokens.length > 0) {
        alert(
          "Äänestämättä jäivät koodit:\n" + data.not_voted_tokens.join("\n")
        );
      }
    })
    .catch(() =>
      alert("Äänestyksen sulkeminen ei ehkä onnistunut! Päivitä sivu!")
    );
}

function openVoting(votingId, is_transferable) {
  const data = {
    is_transferable,
  };
  callApi(`${SITE_ROOT_PATH}admin/votings/${votingId}/open/`, "POST", data)
    .then(() => refreshVotingList(true))
    .catch(() =>
      alert("Äänestyksen avaaminen ei ehkä onnistunut! Päivitä sivu!")
    );
}

// Hakutaulukon funktioi

function searchFunction() {
  // Declare variables
  var input, filter, table, tr, td, i;
  input = document.getElementById("search");
  filter = input.value.toUpperCase();
  table = document.getElementById("search-table");
  tr = table.getElementsByTagName("tr");

  // Loop through all table rows, and hide those who don't match the search query
  for (i = 0; i < tr.length; i++) {
    td = tr[i].getElementsByTagName("td")[0];
    if (td) {
      if (td.innerHTML.toUpperCase().indexOf(filter) > -1) {
        tr[i].style.display = "";
      } else {
        tr[i].style.display = "none";
      }
    }
  }
}

function invalidateActiveTokens() {
  callApi(`${SITE_ROOT_PATH}admin/tokens/invalidate/all/`, "POST")
    .then(() => location.reload())
    .catch(() => alert("Koodien mitätöinti epäonnistui!"));
}

function stopAllSounds() {
  document.querySelectorAll(".sound-track").forEach((track) => {
    track.pause();
    track.currentTime = 0;
  });
}

function playSound(trackNo) {
  var track = document.getElementById(`sound-track-${trackNo}`);

  stopAllSounds();

  if (SOUND_STATE !== trackNo) {
    track.play();
    SOUND_STATE = trackNo;
  } else {
    SOUND_STATE = 0;
  }
}

function clearVotes(votingId) {
  const form = getVotingForm(votingId);
  Array.from(form.querySelectorAll(".voting-order")).forEach(
    (elem) => (elem.innerHTML = "-")
  );
  votesGiven = 0;
  currentVotingId = -1;
}

function setupEventListeners() {
  var transferVoteCandidates = document.getElementsByClassName(
    "transfer-vote-candidate"
  );

  Array.from(transferVoteCandidates).forEach((candidate) => {
    candidate.addEventListener("click", (e) => {
      const candidate = e.target.value;
      const votingId = e.target.value.split("-")[2];
      const form = getVotingForm(votingId);
      const candidateCount = form.querySelectorAll("label").length;

      const getOrderLabel = () => document.getElementById(candidate).innerHTML;
      var label = getOrderLabel();

      if (label === "-") {
        if (currentVotingId !== votingId) {
          currentVotingId = votingId;
          votesGiven = 0;
        } else if (votesGiven < candidateCount) {
          votesGiven += 1;
          document.getElementById(candidate).innerHTML = votesGiven;
        }
      } else {
        const clickedRank = e.target.previousElementSibling.innerHTML;
        e.target.previousElementSibling.innerHTML = "-";
        Array.from(form.querySelectorAll(".voting-order")).forEach((elem) => {
          const rank = parseInt(elem.innerHTML);
          if (rank > clickedRank) {
            elem.innerHTML = rank - 1;
          }
        });
        votesGiven -= 1;
      }
    });
  });
}

window.addEventListener("load", async function () {
  M.Modal.init(document.querySelector("#voting-modal"));
  const token = localStorage.getItem("token");
  if (token) {
    await refreshVotingList();
  }
  setupEventListeners();
});

function expandResults(element) {
  element.classList.toggle("expanded");
}
