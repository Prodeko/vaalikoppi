// Kaikenlaisia funktioita äänestykseen
// Tarvii jQueryn

SITE_ROOT_PATH = "/vaalikoppi/";
SOUND_STATE = 0;

function vote(votingId) {
  var form = $("#voting-form-" + votingId);
  var maxVotes;
  var chosenCandidates = [];

  try {
    maxVotes = parseInt(form.attr("data-voting-max-votes"));
  } catch (err) {
    alert("Äänestyksen tiedot eivät ole latautuneet oikein. Päivitä sivu.");
    return;
  }

  form.find("input[name=candidate]:checked").each(function() {
    var curId = $(this).attr("value");
    var curName = form
      .find("label[for=candidate-v-" + votingId + "-" + curId + "]")
      .text();
    chosenCandidates.push({ id: curId, name: curName });
  });

  var confirmation = confirm(
    "Olet äänestämässä " +
      (chosenCandidates.length > 1 ? "ehdokkaita:\n" : "ehdokasta:\n") +
      chosenCandidates
        .map(function(candi) {
          return candi.name;
        })
        .join(", ")
  );

  if (!confirmation) {
    return;
  }

  form.find("input, button").prop("disabled", true);

  var query = $.post(SITE_ROOT_PATH + "votings/" + votingId + "/vote/", {
    candidates: chosenCandidates.map(function(candi) {
      return candi.id;
    })
  })
    .done(function(data) {
      $("#voting-list-area").html(data);
    })
    .fail(function(data) {
      alert("Äänestäminen epäonnistui. Päivitä sivu ja yritä uudelleen!");
      refreshVotingList();
    });
}

function voteTransferableElection(votingId) {
  var form = $("#voting-form-" + votingId);
  var maxVotes;
  var chosenCandidates = [];

  form.find("label").addClass("disabled");

  try {
    maxVotes = parseInt(form.attr("data-voting-max-votes"));
  } catch (err) {
    alert("Äänestyksen tiedot eivät ole latautuneet oikein. Päivitä sivu.");
    return;
  }

  form.find(".voting-order").each(function() {
    var curId = $(this).attr("value");
    var curName = form
      .find("label[value=candidate-v-" + votingId + "-" + curId + "]")
      .text();
    var position = $(this).text();
    chosenCandidates.push({ id: curId, name: curName, position: position });
  });
  // console.log(chosenCandidates);

  chosenCandidates = chosenCandidates.sort(compareChosenCandidates);

  var postData = [];
  var postData2 = new Map();

  chosenCandidates.map(function(candi) {
    postData2.set(candi.id, candi.position);
  });

  chosenCandidates.forEach(function(candi) {
    postData.push([candi.id, candi.position]);
  });
  //  console.log(postData);

  // TODO: Remove comma separators in confirmation modal
  var confirmation = confirm(
    "Olet äänestämässä " +
      (chosenCandidates.length > 1 ? "ehdokkaita:\n" : "ehdokasta:\n") +
      chosenCandidates
        .filter(candi => candi.position != "-")
        .map(candi => candi.position + ". " + candi.name)
        .join(", ")
  );

  if (!confirmation) {
    return;
  }

  form.find("input, button").prop("disabled", true);

  var query = $.post(
    SITE_ROOT_PATH + "votings/" + votingId + "/voteTransferable/",
    {
      candidates: chosenCandidates.map(function(candi) {
        return candi.id + ":" + candi.position;
      })
    }
  )
    .done(function(data) {
      $("#voting-list-area").html(data);
    })
    .fail(function(data) {
      alert("Äänestäminen epäonnistui. Päivitä sivu ja yritä uudelleen!");
      refreshVotingList();
    });
}

// used to sort candidates by position in transferable election confirmation modal
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

function refreshVotingList(admin = false) {
  var votingArea = $("#voting-list-area");

  var query = $.get(
    SITE_ROOT_PATH + (admin ? "admin/" : "") + "votings/list/",
    function(data) {
      votingArea.html(data);
    }
  ).fail(function() {
    alert(
      "Äänestysten haku ei onnistunut. Päivitä sivu. Jos koetit äänestää, katso, näkyykö äänestys jo äänestettynä."
    );
  });
}

function checkboxClick(votingId, candidateId) {
  var form = $("#voting-form-" + votingId);
  var maxVotes = parseInt(form.attr("data-voting-max-votes"));

  if (form.find("input[type=checkbox]:checked").length > maxVotes) {
    form
      .find("#candidate-v-" + votingId + "-" + candidateId)
      .prop("checked", false);
  }
}

function generateTokens(count) {
  $("#generate-tokens-button").prop("disabled", true);

  var query = $.post(SITE_ROOT_PATH + "admin/tokens/generate/", {
    count: count
  })
    .done(function(data) {
      alert("Koodien generointi onnistui.");
      location.reload();
    })
    .fail(function(data) {
      alert("Koodien generointi epäonnistui.");
    });

  $("#generate-tokens-button").prop("disabled", false);
}

function invalidateToken(code, number) {
  var invalidateButton = $("#invalidate-token-button-" + number);
  var clickedState = invalidateButton.data("clicked");

  // Require two clicks to activate code
  if (clickedState == "0") {
    invalidateButton.html("Mitätöi?");
    invalidateButton.addClass("orange");
    invalidateButton.removeClass("red");
    invalidateButton.data("clicked", "1");
    return;
  }

  var token = code;

  if (token.length < 1) {
    return;
  }

  invalidateButton.prop("disabled", true);

  var query = $.post(SITE_ROOT_PATH + "admin/tokens/invalidate/", {
    token: token
  })
    .done(function(data) {
      // alert('Koodin mitätöinti onnistui.');
      location.reload();
    })
    .fail(function(data) {
      alert("Koodin mitätöinti epäonnistui. Tarkista koodi.");
    });
}

function activateToken(code, number) {
  var activateButton = $("#activate-token-button-" + number);
  var clickedState = activateButton.data("clicked");

  // Require two clicks to activate code
  if (clickedState == "0") {
    activateButton.html("Aktivoi?");
    activateButton.addClass("orange");
    activateButton.removeClass("green");
    activateButton.data("clicked", "1");
    return;
  }

  var token = code;

  if (token.length < 1) {
    return;
  }

  activateButton.prop("disabled", true);

  var query = $.post(SITE_ROOT_PATH + "admin/tokens/activate/", {
    token: token
  })
    .done(function(data) {
      //alert('Koodin aktivointi onnistui.');
      location.reload();
    })
    .fail(function(data) {
      alert("Koodin aktivointi epäonnistui. Tarkista koodi.");
    });
}

// Just for the UI. Everything is validated in the back-end...
function checkVoterStatus(callback) {
  var query = $.getJSON(SITE_ROOT_PATH + "user/status/")
    .done(function(data) {
      try {
        // No token/non-active token
        if (data.status === 0) {
          callback(false);
        } else if (data.status === 1) {
          $("#token_div").html(data.token);
          callback(true);
        } else {
          throw new Exception();
        }
      } catch (err) {
        callback(false);
      }
    })
    .fail(function() {
      alert("Tilan haku palvelimelta ei onnistunut. Koeta päivittää sivu.");
    });
}

function logout() {
  var query = $.post(SITE_ROOT_PATH + "user/logout/")
    .done(function(data) {
      if (data.status === 0) {
        document.cookie = "";
        location.reload(true);
      }
    })
    .fail(function(data) {
      alert("Uloskirjautuminen epäonnistui. Päivitä sivu.");
    });
}

function submitToken() {
  var token = $("#type-token-field").val();
  var notificationArea = $("#login-notification-area");

  notificationArea.removeClass("wrong-token-warning");
  notificationArea.addClass("loading-token-notification");
  notificationArea.html("Ladataan...");

  var query = $.post(SITE_ROOT_PATH + "user/login/", { token: token })
    .done(function(data) {
      /* toggleLoginPrompt();
		// Below adds the token to the top bar
		checkVoterStatus();
		refreshVotingList(); */
      location.reload();
    })
    .fail(function(data) {
      window.setTimeout(function() {
        notificationArea.removeClass("loading-token-notification");
        notificationArea.addClass("wrong-token-warning");
        notificationArea.html("Virheellinen koodi");
      }, 100);
    });
}

function toggleLoginPrompt() {
  $("#main-login-prompt").toggle();
  $("#voting-list-updater, #voting-list-area").fadeToggle();
}

function create_voting() {
  const is_transferable = $("#is_transfer_election").is(":checked");
  const voting_name = $("#voting_name").val();
  const voting_description = $("#voting_description").val();
  const max_votes = $("#max_votes").val() ? $("#max_votes").val() : 1;
  console.log("is_transferable: " + is_transferable);

  $.post(SITE_ROOT_PATH + "admin/votings/create/", {
    is_transferable: is_transferable,
    voting_name: voting_name,
    voting_description: voting_description,
    max_votes: max_votes
  })
    .done(function(data) {
      refreshVotingList(true); // TEMP CHANGED TO TRANSFERABLE VOTES
    })
    .fail(function(data) {
      alert("Äänestyksen luominen ei ehkä onnistunut! Päivitä sivu!");
    });
}

function add_candidate(voting_id, is_transferable) {
  var candidate = $("#voting-" + voting_id + "-candidate_name").val();
  if (candidate) {
    $.post(SITE_ROOT_PATH + "admin/votings/" + voting_id + "/add/", {
      is_transferable: is_transferable,
      candidate_name: candidate
    })
      .done(function(data) {
        refreshVotingList(true);
      })
      .fail(function(data) {
        alert("Ehdokkaan lisääminen ei ehkä onnistunut! Päivitä sivu!");
      });
  }
}

function remove_candidate(candidate_id, is_transferable) {
  $.post(SITE_ROOT_PATH + "admin/votings/" + candidate_id + "/remove/", {
    is_transferable: is_transferable
  })
    .done(function(data) {
      refreshVotingList(true);
    })
    .fail(function(data) {
      alert("Äänestyksen luominen ei ehkä onnistunut! Päivitä sivu!");
    });
}

function closeVoting(votingId, is_transferable) {
  $.post(SITE_ROOT_PATH + "admin/votings/" + votingId + "/close/", {
    is_transferable: is_transferable
  })
    .done(function(data) {
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
    .fail(function(data) {
      alert("Äänestyksen sulkeminen ei ehkä onnistunut! Päivitä sivu!");
    });
}

function openVoting(votingId, is_transferable) {
  $.post(SITE_ROOT_PATH + "admin/votings/" + votingId + "/open/", {
    is_transferable: is_transferable
  })
    .done(function(data) {
      refreshVotingList(true); // TEMP CHANGED TO TRANSFERABLE VOTES
    })
    .fail(function(data) {
      alert("Äänestyksen avaaminen ei ehkä onnistunut! Päivitä sivu!");
    });
}

function openVotingTransferable(votingId) {
  var query = $.post(
    SITE_ROOT_PATH + "admin/votings/" + votingId + "/openTransferable/"
  )
    .done(function(data) {
      refreshVotingList(true); // TEMP CHANGED TO TRANSFERABLE VOTES
    })
    .fail(function(data) {
      alert("Äänestyksen avaaminen ei ehkä onnistunut! Päivitä sivu!");
    });
}

// Hakutaulukon funktioi

function searchFunction() {
  // Declare variables
  var input, filter, table, tr, td, i;
  input = document.getElementById("searchInput");
  filter = input.value.toUpperCase();
  table = document.getElementById("searchTable");
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
  var query = $.post(SITE_ROOT_PATH + "admin/tokens/invalidate/all/")
    .done(function(data) {
      location.reload();
    })
    .fail(function(data) {
      alert("Koodien mitätöinti epäonnistui!");
    });
}

function stopAllSounds() {
  $(".sound-track").each(function() {
    $(this)
      .get(0)
      .pause();
    $(this).get(0).currentTime = 0;
  });
}

function playSound(trackNo) {
  var tracks = ["drums", "doubling", "badumtss"];
  var chosenTrack = tracks[trackNo - 1];
  var trackEle = $("#sound-track-" + trackNo).get(0);

  stopAllSounds();

  if (SOUND_STATE !== trackNo) {
    trackEle.play();
    SOUND_STATE = trackNo;
  } else {
    SOUND_STATE = 0;
  }
}
$(document).ready(function() {
  var currentVotingId = -1;
  var votes = [];
  var votesGiven = 0;

  $(document).on("click", ".transfer-vote-candidate", function() {
    const candidate = $(this).attr("value");
    const voting = $(this).parents(".voting-form-transferable")[0];
    const votingID = $(voting)
      .attr("id")
      .replace("voting-form-", "");
    const candidateCount = $(voting).find("label").length;

    if ($("#" + candidate).text() == "-") {
      if (currentVotingId != votingID) {
        votes = new Array();
        currentVotingId = votingID;
        votesGiven = 0;
      }
      if (votesGiven < candidateCount) {
        votesGiven += 1;
        $("#" + candidate).text(votesGiven);
      }
    } else {
      const value = parseInt($("#" + candidate).text());
      $("#" + candidate).text("-");
      $(voting)
        .find(".voting-order")
        .each(function() {
          const rank = parseInt($(this).text());
          if (rank > value) {
            return $(this).text(rank - 1);
          } else {
            return this;
          }
        });
      votesGiven -= 1;
    }
  });

  $(document).on("click", ".clear-vote", function() {
    var votingId = $(this)
      .parent()
      .attr("id")
      .substr(12);

    $(this)
      .parents(".voting-form-transferable")
      .find(".voting-order")
      .each(function() {
        $(this).text("-");
      });
    votes = new Array();
    votesGiven = 0;
    currentVotingId = -1;
  });
});

function expandResults(element) {
  $(element).toggleClass("expanded");
}
